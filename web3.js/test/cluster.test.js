// @flow
import {clusterApiUrl} from '../src/util/cluster';

test('invalid', () => {
  expect(() => {
    // $FlowExpectedError
    clusterApiUrl('abc123');
  }).toThrow();
});

test('devnet', () => {
  expect(clusterApiUrl()).toEqual('https://devnet.panoptes.org');
  expect(clusterApiUrl('devnet')).toEqual('https://devnet.panoptes.org');
  expect(clusterApiUrl('devnet', true)).toEqual('https://devnet.panoptes.org');
  expect(clusterApiUrl('devnet', false)).toEqual('http://devnet.panoptes.org');
});
