import { GraphQLClient } from 'graphql-request';
import { getSdk, User } from './generated/graphql';
import * as React from 'react';
import * as ReactDOM from 'react-dom';

const client = new GraphQLClient('http://localhost:3000/graphql');
const sdk = getSdk(client);

function App() {
  const [userName, setUserName] = React.useState<string | null>(null);
  const [userId, setUserId] = React.useState<number | null>(null);

  const onChange = (evt: React.ChangeEvent<HTMLInputElement>) => {
    const id = parseInt(evt.target.value, 10);
    if (!isNaN(id)) {
      setUserId(id);
      setUserName(null);
      sdk.getUser({id}).then((u) => setUserName(u.user ? u.user.name : ''));
    }
  };

  const input = <input type='number' value={userId ? userId : undefined} onChange={onChange}></input>;
  return (<>
    {input}
    {renderUser(userName)}
  </>);
}

function renderUser(userName: string | null) {
  if (userName == null) {
    return <div>Loading...</div>;
  } else if (userName === '') {
    return <div>Not found</div>;
  } else {
    return <div>User: {userName}</div>;
  }
}

ReactDOM.render(<App/>, document.getElementById('main')!);
