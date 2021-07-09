import { GraphQLClient } from 'graphql-request';
import { getSdk } from './generated/graphql';
import * as React from 'react';
import * as ReactDOM from 'react-dom';
import {
  createTheme,
  Box,
  Card,
  CardContent,
  CardHeader,
  Container,
  CssBaseline,
  Grid,
  TextField,
  ThemeProvider,
} from '@material-ui/core';

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

  const theme = createTheme({
    palette: {
      type: 'dark',
    },
  });
  const input = <TextField type='number' label='User ID' value={userId == null ? '' : userId} onChange={onChange}></TextField>;
  return (
    <ThemeProvider theme={theme}>
      <CssBaseline/>
      <Container>
        <Box my={4}>
        <Grid container>
          <Grid item xs>
          <form>
            {input}
          </form>
        </Grid>
        <Grid item xs>
          <Card>
            <CardHeader title='User'/>
            <CardContent>
              {renderUser(userName)}
            </CardContent>
          </Card>
        </Grid>
        </Grid>
        </Box>
      </Container>
    </ThemeProvider>
  );
}

function renderUser(userName: string | null) {
  if (userName == null) {
    return <div>Loading...</div>;
  } else if (userName === '') {
    return <div>Not found</div>;
  } else {
    return <div>{userName}</div>;
  }
}

ReactDOM.render(<App/>, document.getElementById('main')!);
