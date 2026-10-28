import type { SearchRadiusKm } from '@/stores/onboarding.store'

export interface DemoBarber {
  id: string
  name: string
  distance: number
  rating: number
  specialty: string
  secondarySpecialty: string
  available?: boolean
  image: string
  imageAlt: string
}

export type FilterSheet =
  | 'location'
  | 'distance'
  | 'rating'
  | 'services'
  | 'availability'
  | 'all'
  | null

export type FilterSheetName = Exclude<FilterSheet, null>
export type { SearchRadiusKm }

export type LocationStatus =
  | 'idle'
  | 'requesting'
  | 'success'
  | 'denied'
  | 'disabled'
  | 'unavailable'
