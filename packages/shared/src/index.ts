/** A volume (tome) in the library */
export interface Volume {
  id: string;
  title: string;
  coverUrl: string;
  pageCount: number;
}

/** A single page (or spread) within a volume */
export interface Page {
  filename: string;
  url: string;
  /** true if this is a double-page spread (e.g. 006-007.jpg) */
  isSpread: boolean;
}

/** Reading progress for a volume */
export interface ReadingProgress {
  volumeId: string;
  pageIndex: number;
  timestamp: number;
}
