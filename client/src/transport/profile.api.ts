import { api } from './http'
import type { ApiResponse } from './envelopes' 
import type { AvatarSignatureResponse, UpdateAvatarRequest } from './profile.contracts'

export const profileApi = {
  async getSignature(): Promise<ApiResponse<AvatarSignatureResponse>> {
    return api.get<AvatarSignatureResponse>('/client/avatar/signature')
  },

  async saveAvatar(payload: UpdateAvatarRequest): Promise<ApiResponse<void>> {
    console.log('Saving avatar with payload:', payload)
    return api.post<void>('/client/avatar', payload)
  },
}