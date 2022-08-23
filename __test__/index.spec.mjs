import test from 'ava';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);

const __dirname = path.dirname(__filename);

import { resolve } from '../index.js';

test('resolve buildins', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: ['tsconfig.json'] }), {
    found: true,
    path: '',
  });
});

test('resolve buildins when no project provide', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: [] }), {
    found: true,
    path: '',
  });
});

test('resolve buildins with wrong tsconfig', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: ['tsconfig.dummy.json'] }), {
    found: true,
    path: '',
  });
});

test('resolve relative path with tsconfig', (t) => {
  console.log('__dirname', __dirname);
  console.log(path.join(__dirname, '../fixtures/withoutPaths/index.ts'));
  t.deepEqual(
    resolve('./tsImportee', path.join(__dirname, '../fixtures/withoutPaths/index.ts'), { project: [path.join(__dirname, '../'tsconfig.json')] }),
    {
      found: true,
      path: path.resolve(path.join(__dirname, '../fixtures/withoutPaths/tsImportee.ts')),
    },
  );
});
