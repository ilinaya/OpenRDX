import { Injectable } from '@angular/core';
import {HttpClient, HttpParams} from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { User } from '../models/user.model';
import {PagedResponse, PaginationParams} from "../models/pagination.model";
import {UserGroup} from "../models/user-group.model";

@Injectable({
  providedIn: 'root'
})
export class UserService {
  private apiUrl = `${environment.apiUrl}/users`;

  constructor(private http: HttpClient) {}

  getUsers(page: number = 1, pageSize: number = 10): Observable<{ count: number; results: User[] }> {
    return this.http.get<{ count: number; results: User[] }>(`${this.apiUrl}/users/`, {
      params: { page: page.toString(), page_size: pageSize.toString() }
    });
  }

  getUserGroups(params: PaginationParams): Observable<PagedResponse<UserGroup>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<UserGroup>>(`${this.apiUrl}/groups/`, { params: httpParams });
  }

  getUserGroupList(): Observable<UserGroup[]> {
    return this.http.get<UserGroup[]>(`${this.apiUrl}/groups/list_all/`);
  }

  deleteUserGroup(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/groups/${id}/`);
  }

  getUserGroup(id: number): Observable<UserGroup> {
    return this.http.get<UserGroup>(`${this.apiUrl}/groups/${id}/`);
  }

  createUserGroup(group: Partial<UserGroup>): Observable<UserGroup> {
    return this.http.post<UserGroup>(`${this.apiUrl}/groups/`, group);
  }

  updateUserGroup(id: number, group: Partial<UserGroup>): Observable<UserGroup> {
    return this.http.patch<UserGroup>(`${this.apiUrl}/groups/${id}/`, group);
  }

  getUser(id: number): Observable<User> {
    return this.http.get<User>(`${this.apiUrl}/users/${id}/`);
  }

  createUser(user: Partial<User>): Observable<User> {
    return this.http.post<User>(`${this.apiUrl}/users/`, user);
  }

  updateUser(id: number, user: Partial<User>): Observable<User> {
    return this.http.patch<User>(`${this.apiUrl}/users/${id}/`, user);
  }

  deleteUser(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/users/${id}/`);
  }

  // Excel Template Download and Upload for User Groups
  downloadUserGroupTemplate(): Observable<Blob> {
    return this.http.get(`${this.apiUrl}/groups/download_template/`, {
      responseType: 'blob'
    });
  }

  uploadUserGroupsExcel(file: File): Observable<any> {
    const formData = new FormData();
    formData.append('file', file);
    return this.http.post(`${this.apiUrl}/groups/upload_excel/`, formData);
  }

  // Excel Template Download and Upload for Users
  downloadUserTemplate(): Observable<Blob> {
    return this.http.get(`${this.apiUrl}/users/download_template/`, {
      responseType: 'blob'
    });
  }

  uploadUsersExcel(file: File): Observable<any> {
    const formData = new FormData();
    formData.append('file', file);
    return this.http.post(`${this.apiUrl}/users/upload_excel/`, formData);
  }
}
