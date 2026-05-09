/**
 * Adobe 风格 SVG 图标集 | 不引入依赖 | 16px 基准
 */

const SIZE = 16;

function Icon({ children, className = "" }) {
  return (
    <svg
      width={SIZE} height={SIZE} viewBox="0 0 24 24"
      fill="none" stroke="currentColor" strokeWidth="1.8"
      strokeLinecap="round" strokeLinejoin="round"
      className={className}
      aria-hidden="true"
    >
      {children}
    </svg>
  );
}

export function IconChart() {
  return <Icon><path d="M3 3v18h18"/><path d="M7 16l4-8 4 4 4-6"/></Icon>;
}

export function IconCode() {
  return <Icon><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></Icon>;
}

export function IconCheck() {
  return <Icon><polyline points="20 6 9 17 4 12"/></Icon>;
}

export function IconAlert() {
  return <Icon><path d="M12 2L2 22h20L12 2z"/><line x1="12" y1="10" x2="12" y2="14"/><line x1="12" y1="18" x2="12.01" y2="18"/></Icon>;
}

export function IconCamera() {
  return <Icon><path d="M23 7l-4-4H5L1 7v11a2 2 0 002 2h18a2 2 0 002-2V7z"/><circle cx="12" cy="13" r="3"/></Icon>;
}

export function IconBook() {
  return <Icon><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M4 4.5A2.5 2.5 0 016.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15z"/></Icon>;
}

export function IconFlask() {
  return <Icon><path d="M9 3h6v5l4 10H5l4-10V3z"/><line x1="12" y1="11" x2="12" y2="15"/></Icon>;
}

export function IconPlay() {
  return <Icon><polygon points="5 3 19 12 5 21 5 3"/></Icon>;
}

export function IconRefresh() {
  return <Icon><polyline points="1 4 1 10 7 10"/><path d="M21 20v-6h-6"/><path d="M20.49 9A9 9 0 005.64 5.64L1 10"/></Icon>;
}

export function IconSettings() {
  return <Icon><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></Icon>;
}

export function IconSearch() {
  return <Icon><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></Icon>;
}

export function IconPlus() {
  return <Icon><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></Icon>;
}

export function IconTrash() {
  return <Icon><polyline points="3 6 5 6 21 6"/><path d="M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6"/><line x1="10" y1="11" x2="10" y2="17"/><line x1="14" y1="11" x2="14" y2="17"/></Icon>;
}

export function IconCompile() {
  return <Icon><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></Icon>;
}

export function IconHome() {
  return <Icon><path d="M3 12l9-9 9 9"/><path d="M5 10v10a1 1 0 001 1h3v-4a2 2 0 014 0v4h3a1 1 0 001-1V10"/></Icon>;
}

export function IconChevronLeft() {
  return <Icon><polyline points="15 18 9 12 15 6"/></Icon>;
}

export function IconChevronRight() {
  return <Icon><polyline points="9 18 15 12 9 6"/></Icon>;
}

export function IconX() {
  return <Icon><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></Icon>;
}

export default Icon;
