/** 品牌标记：三根递增的柱子，对应「字数趋势」 */

interface LogoProps {
  size?: number;
}

export function Logo({ size = 14 }: LogoProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 14 14" aria-hidden="true">
      <g fill="#fff">
        <rect x="1.4" y="7.6" width="2.6" height="5" rx="1" />
        <rect x="5.7" y="4.4" width="2.6" height="8.2" rx="1" />
        <rect x="10" y="1.4" width="2.6" height="11.2" rx="1" />
      </g>
    </svg>
  );
}
