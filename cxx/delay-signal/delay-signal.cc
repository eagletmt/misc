#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/epoll.h>
#include <sys/signalfd.h>
#include <sys/timerfd.h>
#include <sys/wait.h>
#include <unistd.h>
#include <map>

static void die(const char *msg) {
  perror(msg);
  exit(EXIT_FAILURE);
}

static int setup_signalfd(int epoll_fd, sigset_t *mask) {
  int fd = signalfd(-1, mask, SFD_CLOEXEC);
  if (fd == -1) {
    perror("signalfd");
  }
  epoll_event event = {
      .events = EPOLLIN, .data = {.fd = fd},
  };
  if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, fd, &event) == -1) {
    perror("epoll_ctl(ADD, signalfd)");
  }
  return fd;
}

int main(int argc, char *argv[]) {
  if (argc < 2) {
    fprintf(stderr, "Usage: %s COMMAND ARG ...\n", argv[0]);
    return 1;
  }
  long delay_sec = 5;

  sigset_t mask, target_mask;
  sigfillset(&mask);
  sigemptyset(&target_mask);
  sigaddset(&target_mask, SIGINT);
  sigaddset(&target_mask, SIGTERM);

  if (sigprocmask(SIG_BLOCK, &mask, NULL) == -1) {
    die("sigprocmask");
  }

  const int epoll_fd = epoll_create1(EPOLL_CLOEXEC);
  if (epoll_fd == -1) {
    die("epoll_create1");
  }
  const int sig_fd = setup_signalfd(epoll_fd, &mask);

  pid_t pid = fork();
  if (pid == -1) {
    die("fork");
  } else if (pid == 0) {
    sigprocmask(SIG_UNBLOCK, &mask, NULL);
    setsid();
    execvp(argv[1], argv + 1);
    die("execvp");
  }

  std::map<int, int> delayed_signo;
  int child_status;

  for (;;) {
    epoll_event event;
    if (epoll_wait(epoll_fd, &event, 1, -1) == -1) {
      die("epoll_wait");
    }

    if (event.data.fd == sig_fd) {
      signalfd_siginfo info;
      ssize_t n = read(event.data.fd, &info, sizeof(info));
      if (n != sizeof(info)) {
        die("read");
      }

      if (info.ssi_signo == SIGCHLD) {
        pid_t p = waitpid(pid, &child_status, WNOHANG);
        if (p == -1) {
          die("waitpid");
        } else if (p == pid) {
          break;
        }
      } else {
        if (sigismember(&target_mask, info.ssi_signo)) {
          int timer_fd = timerfd_create(CLOCK_MONOTONIC, TFD_CLOEXEC);
          itimerspec it = {
              .it_interval =
                  {
                      .tv_sec = 0, .tv_nsec = 0,
                  },
              .it_value =
                  {
                      .tv_sec = delay_sec, .tv_nsec = 0,
                  },
          };
          timerfd_settime(timer_fd, 0, &it, NULL);
          epoll_event ev = {
              .events = EPOLLIN, .data = {.fd = timer_fd},
          };
          epoll_ctl(epoll_fd, EPOLL_CTL_ADD, timer_fd, &ev);
          delayed_signo.insert(std::make_pair(timer_fd, info.ssi_signo));
        } else {
          kill(pid, info.ssi_signo);
        }
      }
    } else {
      auto it = delayed_signo.find(event.data.fd);
      kill(pid, it->second);
      epoll_ctl(epoll_fd, EPOLL_CTL_DEL, it->first, NULL);
      close(it->first);
      delayed_signo.erase(it);
    }
  }

  close(epoll_fd);
  for (const auto &p : delayed_signo) {
    close(p.first);
  }
  close(sig_fd);

  return WEXITSTATUS(child_status);
}
