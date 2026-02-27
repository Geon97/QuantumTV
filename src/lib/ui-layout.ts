export const appLayoutClasses = {
  pageShell:
    'mx-auto w-full max-w-[1720px] px-3 max-[375px]:px-2.5 min-[834px]:px-7 min-[1440px]:px-10',
  pageContent:
    'mx-auto w-full max-w-[1480px] min-[834px]:max-w-[1560px] min-[1440px]:max-w-[1640px]',
  sectionGap:
    'mb-8 max-[375px]:mb-7 min-[834px]:mb-11 min-[1440px]:mb-14',
} as const;

export type CardGridDensity = 'default' | 'dense';

export function getGridColumnsClass(density: CardGridDensity = 'default') {
  if (density === 'dense') {
    return [
      'grid',
      'grid-cols-2',
      'max-[375px]:grid-cols-2',
      'sm:grid-cols-3',
      'min-[834px]:grid-cols-4',
      'md:grid-cols-4',
      'lg:grid-cols-6',
      'min-[1440px]:grid-cols-7',
      '2xl:grid-cols-8',
      'gap-x-3',
      'gap-y-8',
      'max-[375px]:gap-x-2',
      'max-[375px]:gap-y-7',
      'sm:gap-x-4',
      'sm:gap-y-10',
      'min-[834px]:gap-x-5',
      'min-[834px]:gap-y-11',
      'lg:gap-x-5',
      'lg:gap-y-12',
      'min-[1440px]:gap-x-6',
      'min-[1440px]:gap-y-14',
    ].join(' ');
  }

  return [
    'grid',
    'grid-cols-2',
    'max-[375px]:grid-cols-2',
    'sm:grid-cols-3',
    'min-[834px]:grid-cols-4',
    'md:grid-cols-4',
    'lg:grid-cols-5',
    'xl:grid-cols-6',
    'min-[1440px]:grid-cols-7',
    'gap-x-3',
    'gap-y-8',
    'max-[375px]:gap-x-2',
    'max-[375px]:gap-y-7',
    'sm:gap-x-4',
    'sm:gap-y-10',
    'min-[834px]:gap-x-5',
    'min-[834px]:gap-y-11',
    'lg:gap-x-6',
    'lg:gap-y-12',
    'min-[1440px]:gap-x-7',
    'min-[1440px]:gap-y-14',
  ].join(' ');
}

export type RailItemDensity = 'default' | 'compact';

export function getRailItemClass(density: RailItemDensity = 'default') {
  if (density === 'compact') {
    return [
      'min-w-20',
      'w-20',
      'max-[375px]:min-w-18',
      'max-[375px]:w-18',
      'sm:min-w-24',
      'sm:w-24',
      'min-[834px]:min-w-30',
      'min-[834px]:w-30',
      'md:min-w-32',
      'md:w-32',
      'min-[1440px]:min-w-40',
      'min-[1440px]:w-40',
      'xl:min-w-36',
      'xl:w-36',
    ].join(' ');
  }

  return [
    'min-w-24',
    'w-24',
    'max-[375px]:min-w-20',
    'max-[375px]:w-20',
    'sm:min-w-28',
    'sm:w-28',
    'min-[834px]:min-w-34',
    'min-[834px]:w-34',
    'md:min-w-36',
    'md:w-36',
    'min-[1440px]:min-w-48',
    'min-[1440px]:w-48',
    'xl:min-w-44',
    'xl:w-44',
  ].join(' ');
}
