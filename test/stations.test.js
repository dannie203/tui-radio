import { test, describe } from 'node:test';
import assert from 'node:assert/strict';
import { deduplicateStations, fetchStations } from '../src/api/stations.js';

describe('Radio Stations API & Deduplication', () => {
  test('deduplicates stations with identical name and country, preferring higher bitrate', () => {
    const rawStations = [
      { id: '1', name: 'HOT 97', country: 'United States', bitrate: 64, url: 'http://stream1.net' },
      { id: '2', name: 'HOT 97', country: 'United States', bitrate: 128, url: 'http://stream2.net' },
      { id: '3', name: 'HOT 97', country: 'United States', bitrate: 96, url: 'http://stream3.net' }
    ];

    const result = deduplicateStations(rawStations);
    assert.equal(result.length, 1);
    assert.equal(result[0].name, 'HOT 97');
    assert.equal(result[0].bitrate, 128);
    assert.equal(result[0].url, 'http://stream2.net');
  });

  test('deduplicates identical stream URLs even if names have minor whitespace differences', () => {
    const rawStations = [
      { id: '1', name: 'Rap Radio', country: 'Germany', bitrate: 128, url: 'https://stream.rap.fm/live/' },
      { id: '2', name: 'Rap Radio ', country: 'Germany', bitrate: 128, url: 'https://stream.rap.fm/live' }
    ];

    const result = deduplicateStations(rawStations);
    assert.equal(result.length, 1);
  });

  test('preserves stations with the same name from different countries', () => {
    const rawStations = [
      { id: '1', name: 'Energy Hip Hop', country: 'Germany', bitrate: 128, url: 'http://stream-de.net' },
      { id: '2', name: 'Energy Hip Hop', country: 'France', bitrate: 128, url: 'http://stream-fr.net' }
    ];

    const result = deduplicateStations(rawStations);
    assert.equal(result.length, 2);
    assert.equal(result[0].country, 'Germany');
    assert.equal(result[1].country, 'France');
  });

  test('falls back to local fallback when network request fails', async () => {
    const mockFetch = async () => {
      throw new Error('Network offline');
    };

    const res = await fetchStations({ fetchImpl: mockFetch });
    assert.equal(res.source, 'local fallback');
    assert.ok(res.stations.length > 0);
  });
});
