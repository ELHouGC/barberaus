export interface AvatarSignatureResponse {
  signature: string
  timestamp: number
  api_key: string
  cloud_name: string
  folder: string
}

export interface CropCoordinates {
  x: number
  y: number
  width: number
  height: number
}

export interface UpdateAvatarRequest {
  avatar_url: string
  crop_coordinates: CropCoordinates
}
