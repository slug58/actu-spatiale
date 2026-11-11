import launchesData from '../data/launches.json';
import socialData from '../data/social.json';
import newsData from '../data/news.json';

export type Launch = (typeof launchesData)[number];
export type Article = (typeof newsData)[number];

/** Contrat de site/src/data/social.json (écrit par le robot Rust). */
export interface SocialPost {
  id: string;
  platform: 'bluesky' | 'reddit' | string;
  author: string;
  handle: string;
  avatar: string | null;
  text: string;
  url: string;
  image: string | null;
  publishedAt: string;
  likes: number | null;
  reposts: number | null;
  starship: boolean;
}

export const launches: Launch[] = launchesData;
export const news: Article[] = newsData;
export const social: SocialPost[] = (socialData as unknown as SocialPost[])
  .slice()
  .sort((a, b) => b.publishedAt.localeCompare(a.publishedAt));

/** Construit un lien interne qui respecte le `base` de GitHub Pages (/nom-du-depot/). */
export const url = (path = '') =>
  `${import.meta.env.BASE_URL.replace(/\/$/, '')}/${path.replace(/^\//, '')}`;

export type Tone = 'ok' | 'ko' | 'warn' | 'go' | 'tbd';

export const STATUS: Record<string, { label: string; tone: Tone }> = {
  Success: { label: 'Succès', tone: 'ok' },
  Failure: { label: 'Échec', tone: 'ko' },
  'Partial Failure': { label: 'Échec partiel', tone: 'warn' },
  Go: { label: 'Go', tone: 'go' },
  TBC: { label: 'À confirmer', tone: 'go' },
  TBD: { label: 'Date indicative', tone: 'tbd' },
  Hold: { label: 'En attente', tone: 'warn' },
  'In Flight': { label: 'En vol', tone: 'go' },
};

export const status = (s: string): { label: string; tone: Tone } => STATUS[s] ?? { label: s, tone: 'tbd' };

export const isDone = (l: Launch) => ['Success', 'Failure', 'Partial Failure'].includes(l.status);
export const isUpcoming = (l: Launch) => !isDone(l);

const TZ = 'Europe/Paris';

/** Affiche la date avec la précision réellement connue (heure, jour, mois ou année). */
export function formatNet(net: string, precision?: string | null): string {
  const d = new Date(net);
  const p = precision ?? 'SEC';
  if (['SEC', 'MIN', 'HR'].includes(p))
    return new Intl.DateTimeFormat('fr-FR', { dateStyle: 'long', timeStyle: 'short', timeZone: TZ }).format(d);
  if (p === 'D') return new Intl.DateTimeFormat('fr-FR', { dateStyle: 'long', timeZone: TZ }).format(d);
  if (['W', 'M'].includes(p))
    return 'vers ' + new Intl.DateTimeFormat('fr-FR', { month: 'long', year: 'numeric', timeZone: TZ }).format(d);
  return 'vers ' + d.getUTCFullYear();
}

/** Le compte à rebours n'a de sens que si l'heure est connue à quelques heures près. */
export const hasPreciseTime = (precision?: string | null) => ['SEC', 'MIN', 'HR', 'D', undefined, null].includes(precision as string);

export const formatDate = (d: Date | string) =>
  new Intl.DateTimeFormat('fr-FR', { dateStyle: 'medium', timeZone: TZ }).format(new Date(d));

export const truncate = (s: string | null | undefined, n = 180) => {
  if (!s) return '';
  if (s.length <= n) return s;
  const cut = s.lastIndexOf(' ', n);
  return s.slice(0, cut > 0 ? cut : n) + '…';
};
