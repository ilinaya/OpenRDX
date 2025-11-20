import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable, forkJoin } from 'rxjs';
import { map, switchMap } from 'rxjs/operators';
import { environment } from '../../../environments/environment';
import { Nas, NasCreate, NasUpdate, NasGroup } from '../models/nas.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class NasService {
  private apiUrl = `${environment.apiUrl}/nas`;

  constructor(private http: HttpClient) { }

  // NAS Groups
  getAllNasGroups(params: PaginationParams): Observable<PagedResponse<NasGroup>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<NasGroup>>(`${this.apiUrl}/groups/`, { params: httpParams });
  }

  getNasGroupById(id: number): Observable<NasGroup> {
    return this.http.get<NasGroup>(`${this.apiUrl}/groups/${id}`);
  }

  createNasGroup(group: { name: string; description: string; parent?: number }): Observable<NasGroup> {
    return this.http.post<NasGroup>(`${this.apiUrl}/groups/`, group);
  }

  updateNasGroup(id: number, group: { name?: string; description?: string; parent?: number }): Observable<NasGroup> {
    return this.http.patch<NasGroup>(`${this.apiUrl}/groups/${id}/`, group);
  }

  deleteNasGroup(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/groups/${id}`);
  }

  getNasGroupTree(): Observable<NasGroup[]> {
    return this.http.get<NasGroup[]>(`${this.apiUrl}/groups/tree/`);
  }

  // NAS Devices
  getAllNas(params: PaginationParams): Observable<PagedResponse<Nas>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<Nas>>(`${this.apiUrl}/nas/`, { params: httpParams });
  }

  getNasById(id: number): Observable<Nas> {
    return this.http.get<Nas>(`${this.apiUrl}/nas/${id}/`);
  }

  createNas(nas: NasCreate): Observable<Nas> {
    return this.http.post<Nas>(this.apiUrl, nas);
  }

  updateNas(id: number, nas: NasUpdate): Observable<Nas> {
    return this.http.patch<Nas>(`${this.apiUrl}/nas/${id}`, nas);
  }

  deleteNas(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/nas/${id}`);
  }

  getNasByGroup(groupId: number): Observable<Nas[]> {
    return this.http.get<Nas[]>(`${this.apiUrl}/by_group/`, {
      params: new HttpParams().set('group_id', groupId.toString())
    });
  }

  getAuthorizedNas(userId: number, identifierId: number): Observable<Nas[]> {
    const usersApiUrl = `${environment.apiUrl}/users`;
    return this.http.get<any[]>(`${usersApiUrl}/users/${userId}/identifiers/${identifierId}/nas-authorizations/`).pipe(
      // Transform the response to extract NAS objects from authorization objects
      map((authorizations: any[]) => {
        return authorizations.map((auth: any) => {
          // The authorization object has a nas field which can be an object or just an ID
          if (auth.nas && typeof auth.nas === 'object') {
            return auth.nas;
          } else if (auth.nas_id) {
            // If nas is just an ID, we'll need to fetch it or use nas_name as fallback
            return {
              id: auth.nas_id || auth.nas,
              name: auth.nas_name || `NAS ${auth.nas_id || auth.nas}`,
              // Add other required fields with defaults
              ip_address: '',
              nas_identifier: auth.nas_identifier || '',
              description: '',
              coa_enabled: false,
              coa_port: 3799,
              groups: [],
              secret_id: auth.secret_id || 0,
              vendor_id: auth.vendor_id || 0,
              timezone_id: auth.timezone_id,
              created_at: auth.created_at || new Date().toISOString(),
              updated_at: auth.updated_at || new Date().toISOString()
            } as Nas;
          }
          return auth;
        }).filter((nas: any) => nas !== null && nas !== undefined);
      })
    );
  }

  getAvailableNas(userId: number, identifierId: number): Observable<Nas[]> {
    const usersApiUrl = `${environment.apiUrl}/users`;
    return this.http.get<Nas[]>(`${usersApiUrl}/users/${userId}/identifiers/${identifierId}/available-nas/`);
  }

  authorizeNas(userId: number, identifierId: number, nasId: number): Observable<any> {
    const usersApiUrl = `${environment.apiUrl}/users`;
    return this.http.post(`${usersApiUrl}/users/${userId}/identifiers/${identifierId}/nas-authorizations/`, {
      nas: nasId
    });
  }

  revokeAuthorization(userId: number, identifierId: number, nasId: number): Observable<any> {
    const usersApiUrl = `${environment.apiUrl}/users`;
    // First, get the authorization ID by fetching all authorizations and finding the one with matching nas_id
    return this.http.get<any[]>(`${usersApiUrl}/users/${userId}/identifiers/${identifierId}/nas-authorizations/`).pipe(
      switchMap((authorizations: any[]) => {
        const auth = authorizations.find((a: any) => {
          const authNasId = a.nas?.id || a.nas_id || a.nas;
          return authNasId === nasId;
        });
        if (!auth || !auth.id) {
          throw new Error('Authorization not found');
        }
        const authId = auth.id;
        return this.http.delete(`${usersApiUrl}/users/${userId}/identifiers/${identifierId}/nas-authorizations/${authId}/`);
      })
    );
  }

  authorizeAllNas(userId: number, identifierId: number): Observable<any> {
    // Get all available NAS and authorize each one
    return this.getAvailableNas(userId, identifierId).pipe(
      switchMap((nasList: Nas[]) => {
        if (nasList.length === 0) {
          return forkJoin([]);
        }
        const requests = nasList.map((nas: Nas) => 
          this.authorizeNas(userId, identifierId, nas.id)
        );
        return forkJoin(requests);
      })
    );
  }

  revokeAllAuthorizations(userId: number, identifierId: number): Observable<any> {
    // Get all authorized NAS and revoke each one
    return this.getAuthorizedNas(userId, identifierId).pipe(
      switchMap((nasList: Nas[]) => {
        if (nasList.length === 0) {
          return forkJoin([]);
        }
        const requests = nasList.map((nas: Nas) => 
          this.revokeAuthorization(userId, identifierId, nas.id)
        );
        return forkJoin(requests);
      })
    );
  }

  // Excel Template Download and Upload
  downloadNasGroupTemplate(): Observable<Blob> {
    return this.http.get(`${this.apiUrl}/groups/download_template/`, {
      responseType: 'blob'
    });
  }

  uploadNasGroupsExcel(file: File): Observable<any> {
    const formData = new FormData();
    formData.append('file', file);
    return this.http.post(`${this.apiUrl}/groups/upload_excel/`, formData);
  }

  downloadNasDeviceTemplate(): Observable<Blob> {
    return this.http.get(`${this.apiUrl}/nas/download_template/`, {
      responseType: 'blob'
    });
  }

  uploadNasDevicesExcel(file: File): Observable<any> {
    const formData = new FormData();
    formData.append('file', file);
    return this.http.post(`${this.apiUrl}/nas/upload_excel/`, formData);
  }
}
