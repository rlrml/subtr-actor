// Quality indicator types (016-replay-quality-indicator)

export type QualityCategory = 'good' | 'medium' | 'bad';

export const QUALITY_THRESHOLDS = {
  GOOD: 70,
  MEDIUM: 50,
  WARNING: 70,
} as const;

export interface QualitySubMetrics {
  badFrameCount: number;
  gapCount: number;
  avgVelocityError: number;
}

export interface QualityMetrics {
  score: number;
  category: QualityCategory;
  totalFrames: number;
  analyzedFrames: number;
  badFrameCount: number;
  badFrameRate: number;
  gapCount: number;
  gapFrameCount: number;
  gapRate: number;
  avgVelocityError: number;
  ballQuality: QualitySubMetrics;
  carQuality: QualitySubMetrics;
  calculatedAt: string;
  frameworkVersion: string;
}

export function getQualityCategory(score: number): QualityCategory {
  if (score >= QUALITY_THRESHOLDS.GOOD) return 'good';
  if (score >= QUALITY_THRESHOLDS.MEDIUM) return 'medium';
  return 'bad';
}

export function shouldShowWarning(score: number | null | undefined): boolean {
  if (score === null || score === undefined) return false;
  return score < QUALITY_THRESHOLDS.WARNING;
}
