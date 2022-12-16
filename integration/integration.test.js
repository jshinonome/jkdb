import { spawn } from 'child_process';
import { QConnection } from '../src/index';

const qProcess = spawn('q', ['-p', '1999']);

const q = new QConnection({ port: 1999 });


afterAll(() => qProcess.kill());

test('connect to kdb without credentials', done => {
  q.connect(err => {
    expect(err).toBe(null);
    q.sync('sum til 10', (_, res) => {
      expect(res).toStrictEqual(45);
      done();
    });
  });
});

test('connect to kdb with wrong credentials', done => {
  q.connect(err => {
    expect(err).toBe(null);
    q.sync('.z.pw:{and[x~`test;y~"test"]}', (_, _res) => {
      const qWithWrongCredential = new QConnection({
        port: 1999, user: 'test', password: 'dummy'
      });
      qWithWrongCredential.connect(err => {
        expect(err.message).toBe('ERR_CONNECTION_CLOSED - Wrong Credentials?');
        q.asyn('\\x .z.pw');
        done();
      });
    });
  });
});

test('connect to kdb with correct credentials', done => {
  q.connect(err => {
    expect(err).toBe(null);
    q.sync('.z.pw:{and[x~`test;y~"test"]}', (_, _res) => {
      const qWithCorrectCredential = new QConnection({
        port: 1999, user: 'test', password: 'test'
      });
      qWithCorrectCredential.connect(err => {
        expect(err).toBe(null);
        qWithCorrectCredential.close();
        done();
      });
    });
  });
});

test('lost connection while querying', done => {
  const qProcess = spawn('q', ['-p', '1998']);
  qProcess.stdin.write('.z.D', () => {
    const q = new QConnection({ port: 1998 });
    q.connect(err => {
      expect(err).toBe(null);
      q.sync('exit 0', (err, _res) => {
        expect(err.message).toBe('LOST_CONNECTION');
        done();
        qProcess.kill();
      });
    });
  });
});
