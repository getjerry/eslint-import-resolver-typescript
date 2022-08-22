import test from 'ava';

import { resolve } from '../index.js';

test('resolve buildins', (t) => {
  t.deepEqual(resolve('inspector', '/some-dir', { project: ['tsconfig.json'] }), {
    found: true,
    path: 'inspector',
  });
});
