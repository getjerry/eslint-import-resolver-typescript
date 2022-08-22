import test from 'ava';

import { resolve } from '../index.js';

test('resolve buildins', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: ['tsconfig.json'] }), {
    found: true,
    path: 'inspector',
  });
});


test('resolve buildins when no project provide', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: [] }), {
    found: true,
    path: 'inspector',
  });
});

test('resolve buildins with wrong tsconfig', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: ['tsconfig.dummy.json'] }), {
    found: true,
    path: 'inspector',
  });
});
