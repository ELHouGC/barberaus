import type { DemoBarber } from '@/types/discovery'

/** Illustrative data for discovery UI development; not live search results. */
export const demoBarbers: DemoBarber[] = [
  {
    id: 'karim-cutz',
    name: 'Karim Cutz',
    distance: 1.2,
    rating: 4.9,
    specialty: 'Skin fades',
    secondarySpecialty: 'Beards',
    available: true,
    image:
      'https://images.unsplash.com/photo-1627890458144-4c0c481bf4b8?auto=format&fit=crop&w=700&h=760&q=85',
    imageAlt: 'Portrait of a barber in a dark studio',
  },
  {
    id: 'yassine-studio',
    name: 'Yassine Studio',
    distance: 2.5,
    rating: 4.8,
    specialty: 'Textured cuts',
    secondarySpecialty: 'Tapers',
    image:
      'https://images.unsplash.com/photo-1707068869917-6e5493b27932?auto=format&fit=crop&w=700&h=760&q=85',
    imageAlt: 'Portrait of a bearded barber',
  },
  {
    id: 'nabil-grooming',
    name: 'Nabil Grooming',
    distance: 4.1,
    rating: 4.7,
    specialty: 'Classic cuts',
    secondarySpecialty: 'Hot towel',
    available: true,
    image:
      'https://images.unsplash.com/photo-1598735064309-86f7f4ba875b?auto=format&fit=crop&w=700&h=760&q=85',
    imageAlt: 'Professional barber portrait',
  },
  {
    id: 'atelier-19',
    name: 'Atelier 19',
    distance: 7.8,
    rating: 4.6,
    specialty: 'Creative cuts',
    secondarySpecialty: 'Color',
    image:
      'https://images.unsplash.com/photo-1695619358712-3f19cf62e5df?auto=format&fit=crop&w=700&h=760&q=85',
    imageAlt: 'Barber portrait in monochrome',
  },
]
