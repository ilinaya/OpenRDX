import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
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
    return this.http.get<Nas[]>(`${this.apiUrl}/by_group`, {
      params: new HttpParams().set('group_id', groupId.toString())
    });
  }

  getAuthorizedNas(userId: number, identifierId: number): Observable<Nas[]> {
    return this.http.get<Nas[]>(`${this.apiUrl}/authorized/${userId}/${identifierId}`);
  }

  getAvailableNas(userId: number, identifierId: number): Observable<Nas[]> {
    return this.http.get<Nas[]>(`${this.apiUrl}/available/${userId}/${identifierId}`);
  }

  authorizeNas(userId: number, identifierId: number, nasId: number): Observable<any> {
    return this.http.post(`${this.apiUrl}/authorize/${userId}/${identifierId}/${nasId}`, {});
  }

  revokeAuthorization(userId: number, identifierId: number, nasId: number): Observable<any> {
    return this.http.delete(`${this.apiUrl}/authorize/${userId}/${identifierId}/${nasId}`);
  }

  authorizeAllNas(userId: number, identifierId: number): Observable<any> {
    return this.http.post(`${this.apiUrl}/authorize-all/${userId}/${identifierId}`, {});
  }

  revokeAllAuthorizations(userId: number, identifierId: number): Observable<any> {
    return this.http.delete(`${this.apiUrl}/authorize-all/${userId}/${identifierId}`);
  }
}
