import {
  checkPermissions,
  getCurrentPosition,
  requestPermissions,
} from '@tauri-apps/plugin-geolocation'

export type CurrentLocationResult =
  | {
      status: 'success'
      latitude: number
      longitude: number
      accuracyMeters: number
      timestamp: number
    }
  | {
      status: 'permission-denied'
    }
  | {
      status: 'services-disabled'
    }
  | {
      status: 'unavailable'
    }

type PermissionStatus = Awaited<ReturnType<typeof checkPermissions>>

const LOCATION_SERVICES_DISABLED_MESSAGES = new Set([
  // Current Android plugin implementation.
  'Location services are disabled.',
  // Current iOS plugin implementation.
  'Location services are not enabled.',
])

function getErrorMessage(error: unknown): string | null {
  if (typeof error === 'string') {
    return error.trim()
  }

  if (error instanceof Error) {
    return error.message.trim()
  }

  if (
    typeof error === 'object' &&
    error !== null &&
    'message' in error &&
    typeof error.message === 'string'
  ) {
    return error.message.trim()
  }

  return null
}

function isLocationServicesDisabled(error: unknown): boolean {
  const message = getErrorMessage(error)

  return (
    message !== null &&
    LOCATION_SERVICES_DISABLED_MESSAGES.has(message)
  )
}

function hasLocationPermission(
  permissions: PermissionStatus,
): boolean {
  return (
    permissions.location === 'granted' ||
    permissions.coarseLocation === 'granted'
  )
}

function canRequestPermission(
  permissions: PermissionStatus,
): boolean {
  return [
    permissions.location,
    permissions.coarseLocation,
  ].some(
    (status) =>
      status === 'prompt' ||
      status === 'prompt-with-rationale',
  )
}

export async function getCurrentDeviceLocation(): Promise<CurrentLocationResult> {
  try {
    let permissions = await checkPermissions()

    if (!hasLocationPermission(permissions)) {
      if (!canRequestPermission(permissions)) {
        return { status: 'permission-denied' }
      }

      permissions = await requestPermissions(['location'])
    }

    if (!hasLocationPermission(permissions)) {
      return { status: 'permission-denied' }
    }

    const position = await getCurrentPosition({
      enableHighAccuracy: true,
      timeout: 15_000,
      maximumAge: 60_000,
    })

    return {
      status: 'success',
      latitude: position.coords.latitude,
      longitude: position.coords.longitude,
      accuracyMeters: position.coords.accuracy,
      timestamp: position.timestamp,
    }
  } catch (error) {
    if (isLocationServicesDisabled(error)) {
      return { status: 'services-disabled' }
    }

    console.warn('[location] Device location is unavailable:', error)

    return { status: 'unavailable' }
  }
}
