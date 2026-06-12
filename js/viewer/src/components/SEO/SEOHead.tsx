import { Helmet } from 'react-helmet-async';

export interface SEOHeadProps {
  title: string;
  description: string;
  image?: string;
  url?: string;
  type?: 'website' | 'article' | 'video.other';
  noIndex?: boolean;
  canonicalUrl?: string;
  // Structured data for JSON-LD
  structuredData?: object;
}

const SITE_NAME = 'BallCam.tv';
const DEFAULT_IMAGE = '/og-default.png';
const BASE_URL = typeof window !== 'undefined' ? window.location.origin : 'https://ballcam.tv';

/**
 * SEOHead component for managing page meta tags
 *
 * Usage:
 * ```tsx
 * <SEOHead
 *   title="Match: Team A vs Team B"
 *   description="Watch this exciting Rocket League match..."
 *   image="/og/replay-123.png"
 *   type="video.other"
 * />
 * ```
 */
export function SEOHead({
  title,
  description,
  image = DEFAULT_IMAGE,
  url,
  type = 'website',
  noIndex = false,
  canonicalUrl,
  structuredData,
}: SEOHeadProps) {
  // Truncate title to 60 chars and description to 155 chars for SEO
  const truncatedTitle = title.length > 57 ? `${title.slice(0, 57)}...` : title;
  const truncatedDescription = description.length > 152 ? `${description.slice(0, 152)}...` : description;

  const fullTitle = `${truncatedTitle} | ${SITE_NAME}`;
  const currentUrl = url || (typeof window !== 'undefined' ? window.location.href : '');
  const imageUrl = image.startsWith('http') ? image : `${BASE_URL}${image}`;
  const canonical = canonicalUrl || currentUrl;

  return (
    <Helmet>
      {/* Primary Meta Tags */}
      <title>{fullTitle}</title>
      <meta name="title" content={fullTitle} />
      <meta name="description" content={truncatedDescription} />

      {/* Robots */}
      {noIndex && <meta name="robots" content="noindex, nofollow" />}

      {/* Canonical URL */}
      <link rel="canonical" href={canonical} />

      {/* Open Graph / Facebook */}
      <meta property="og:type" content={type} />
      <meta property="og:url" content={currentUrl} />
      <meta property="og:title" content={fullTitle} />
      <meta property="og:description" content={truncatedDescription} />
      <meta property="og:image" content={imageUrl} />
      <meta property="og:site_name" content={SITE_NAME} />

      {/* Twitter */}
      <meta name="twitter:card" content="summary_large_image" />
      <meta name="twitter:url" content={currentUrl} />
      <meta name="twitter:title" content={fullTitle} />
      <meta name="twitter:description" content={truncatedDescription} />
      <meta name="twitter:image" content={imageUrl} />

      {/* Structured Data */}
      {structuredData && (
        <script type="application/ld+json">
          {JSON.stringify(structuredData)}
        </script>
      )}
    </Helmet>
  );
}

export default SEOHead;
