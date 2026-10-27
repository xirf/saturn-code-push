import { describe, expect, it } from 'vitest'
import {
  comparisonUrl,
  previousPublished,
  type Deployment,
} from '../../app/features/deployments/lib/links'
const record = (id: number, extra: Partial<Deployment> = {}): Deployment => ({
  id,
  release_id: 1,
  key: String(id),
  repository: 'owner/repo',
  commit: String(id).repeat(40),
  source_kind: 'commit',
  source_ref: 'main',
  destination: 'app-store',
  status: 'published',
  build_number: '',
  workflow_run: null,
  created_at: '',
  updated_at: '',
  ...extra,
})
describe('deployment comparisons', () => {
  it('skips failed, other destination, newer and other repository records', () => {
    const current = record(5)
    const base = record(1)
    expect(
      previousPublished(
        [
          record(6),
          current,
          record(4, { status: 'failed' }),
          record(3, { destination: 'testflight' }),
          record(2, { repository: 'other/repo' }),
          base,
        ],
        current,
      ),
    ).toEqual(base)
    expect(comparisonUrl(base, current)).toBe(
      `https://github.com/owner/repo/compare/${base.commit}...${current.commit}`,
    )
  })
  it('requires a distinct record from the same repository', () => {
    const current = record(2)
    expect(comparisonUrl(undefined, current)).toBeUndefined()
    expect(comparisonUrl(current, current)).toBeUndefined()
    expect(comparisonUrl(record(1, { repository: 'other/repo' }), current)).toBeUndefined()
  })
})

describe('release and patch baselines', () => {
  it('never uses a patch as the default baseline for a store release', () => {
    const release = record(1)
    expect(previousPublished([record(2, { patch_id: 10 }), release], record(3))).toEqual(release)
  })
  it('compares a patch within its parent release, falling back to its release baseline', () => {
    const base = record(1)
    const patch = record(2, { patch_id: 10 })
    const current = record(5, { patch_id: 11 })
    expect(
      previousPublished([record(4, { release_id: 2, patch_id: 12 }), patch, base], current),
    ).toEqual(patch)
    expect(previousPublished([record(4, { release_id: 2, patch_id: 12 }), base], current)).toEqual(
      base,
    )
  })
})
