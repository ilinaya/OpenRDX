import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';

export interface NasAuthorization {
  id: number;
  nas_device: number;
  nas_device_name: string;
  attribute_group: number;
  attribute_group_name: string;
  created_at: string;
  updated_at: string;
}

export interface CreateNasAuthorization {
  nas_device: number;
  attribute_group: number;
}

export interface UpdateNasAuthorization {
  attribute_group: number;
}

@Injectable({
  providedIn: 'root'
})
export class UserIdentifierNasAuthService {
  private apiUrl = `${environment.apiUrl}/users`;

  constructor(private http: HttpClient) {}

  getAuthorizations(userId: number, identifierId: number): Observable<NasAuthorization[]> {
    return this.http.get<NasAuthorization[]>(`${this.apiUrl}/${userId}/identifiers/${identifierId}/nas-authorizations/`);
  }

  getAvailableNas(userId: number, identifierId: number): Observable<any[]> {
    return this.http.get<any[]>(`${this.apiUrl}/${userId}/identifiers/${identifierId}/available-nas/`);
  }

  createAuthorization(userId: number, identifierId: number, data: CreateNasAuthorization): Observable<NasAuthorization> {
    return this.http.post<NasAuthorization>(`${this.apiUrl}/${userId}/identifiers/${identifierId}/nas-authorizations/`, data);
  }

  updateAuthorization(userId: number, identifierId: number, authId: number, data: UpdateNasAuthorization): Observable<NasAuthorization> {
    return this.http.patch<NasAuthorization>(`${this.apiUrl}/${userId}/identifiers/${identifierId}/nas-authorizations/${authId}/`, data);
  }

  deleteAuthorization(userId: number, identifierId: number, authId: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${userId}/identifiers/${identifierId}/nas-authorizations/${authId}/`);
  }
} 