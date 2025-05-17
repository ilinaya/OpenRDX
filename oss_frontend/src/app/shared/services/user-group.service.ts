import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { UserGroup, UserGroupCreate, UserGroupUpdate } from '../models/user-group.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class UserGroupService {
  private apiUrl = `${environment.apiUrl}/users/groups`;

  constructor(private http: HttpClient) { }

  getAllGroups(params: PaginationParams): Observable<PagedResponse<UserGroup>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<UserGroup>>(`${this.apiUrl}/`, { params: httpParams });
  }

  getGroup(id: number): Observable<UserGroup> {
    return this.http.get<UserGroup>(`${this.apiUrl}/${id}/`);
  }

  createGroup(group: UserGroupCreate): Observable<UserGroup> {
    return this.http.post<UserGroup>(`${this.apiUrl}/`, group);
  }

  updateGroup(id: number, group: UserGroupUpdate): Observable<UserGroup> {
    return this.http.patch<UserGroup>(`${this.apiUrl}/${id}/`, group);
  }

  deleteGroup(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${id}/`);
  }

  getGroupTree(): Observable<UserGroup[]> {
    return this.http.get<UserGroup[]>(`${this.apiUrl}/tree/`);
  }
} 