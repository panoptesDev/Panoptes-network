//@flow

/**
 * @private
 */
const endpoint = {
  http: {
    devnet: 'http://devnet.panoptes.org',
    testnet: 'http://testnet.panoptes.org',
    'mainnet-beta': 'http://api.mainnet-beta.panoptes.org',
  },
  https: {
    devnet: 'https://devnet.panoptes.org',
    testnet: 'https://testnet.panoptes.org',
    'mainnet-beta': 'https://api.mainnet-beta.panoptes.org',
  },
};

export type Cluster = 'devnet' | 'testnet' | 'mainnet-beta';

/**
 * Retrieves the RPC API URL for the specified cluster
 */
export function clusterApiUrl(cluster?: Cluster, tls?: boolean): string {
  const key = tls === false ? 'http' : 'https';

  if (!cluster) {
    return endpoint[key]['devnet'];
  }

  const url = endpoint[key][cluster];
  if (!url) {
    throw new Error(`Unknown ${key} cluster: ${cluster}`);
  }
  return url;
}
