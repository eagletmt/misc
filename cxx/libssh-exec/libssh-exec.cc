#include <iostream>
#include <libssh/libsshpp.hpp>

int main(int argc, char *argv[]) {
  std::string host, cmd;
  if (argc >= 3) {
    host = argv[1];
    cmd = argv[2];
  } else {
    std::cerr << "Usage: " << argv[0] << " host cmd" << std::endl;
    return 1;
  }

  ssh::Session session;
  session.setOption(SSH_OPTIONS_LOG_VERBOSITY, SSH_LOG_INFO);
  session.setOption(SSH_OPTIONS_HOST, "barkhorn");
  session.optionsParseConfig(NULL);
  // https://red.libssh.org/issues/220
  session.setOption(SSH_OPTIONS_ADD_IDENTITY, "%d/id_ed25519");
  session.connect();

  switch (session.isServerKnown()) {
    case SSH_SERVER_KNOWN_OK:
      break;
    case SSH_SERVER_KNOWN_CHANGED:
      std::cerr << "server_known: KNOWN_CHANGED!" << std::endl;
      return 1;
    case SSH_SERVER_FOUND_OTHER:
      std::cerr << "server_known: FOUND_OTHER!" << std::endl;
      return 1;
    case SSH_SERVER_NOT_KNOWN:
      std::cerr << "server_known: NOT_KNOWN" << std::endl;
      break;
    case SSH_SERVER_FILE_NOT_FOUND:
      std::cerr << "server_known: FILE_NOT_FOUND" << std::endl;
      break;
    default:
      std::cerr << "server_known: ERROR" << std::endl;
      return 1;
  }

  session.userauthNone();
  const int available_auth = session.getAuthList();
  if (available_auth & SSH_AUTH_METHOD_PUBLICKEY) {
    const int auth_rc = session.userauthPublickeyAuto();
    if (auth_rc == SSH_AUTH_ERROR) {
      std::cerr << "publickey auth error" << std::endl;
      return 1;
    } else if (auth_rc == SSH_AUTH_SUCCESS) {
      std::cerr << "successfully authenticated" << std::endl;
    } else {
      std::cerr << "publickey auth failed: " << auth_rc << std::endl;
      return 1;
    }
  }

  ssh::Channel channel(session);
  channel.openSession();
  channel.requestExec(cmd.c_str());

  int nbytes;
  char buf[1024];
  const int timeout_msec = 10 * 1000;
  while ((nbytes = channel.read(buf, sizeof(buf), timeout_msec)) > 0) {
    std::cout.write(buf, nbytes);
  }

  channel.sendEof();
  channel.close();

  return 0;
}
