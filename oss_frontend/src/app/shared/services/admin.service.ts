import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { 
  AdminUser, 
  AdminUserCreate, 
  AdminUserUpdate, 
  AdminGroup, 
  AdminGroupCreate, 
  AdminGroupUpdate 
} from '../models/admin.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class AdminService {
  private apiUrl = `${environment.apiUrl}/admin-users`;

  constructor(private http: HttpClient) { }

  getAllAdmins(params: PaginationParams): Observable<PagedResponse<AdminUser>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<AdminUser>>(`${this.apiUrl}/users/`, { params: httpParams });
  }

  getAdmin(id: number): Observable<AdminUser> {
    return this.http.get<AdminUser>(`${this.apiUrl}/users/${id}/`);
  }

  createAdmin(admin: Partial<AdminUser>): Observable<AdminUser> {
    return this.http.post<AdminUser>(`${this.apiUrl}/users/`, admin);
  }

  updateAdmin(id: number, admin: Partial<AdminUser>): Observable<AdminUser> {
    return this.http.patch<AdminUser>(`${this.apiUrl}/users/${id}/`, admin);
  }

  deleteAdmin(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/users/${id}/`);
  }

  activateAdminUser(id: number): Observable<AdminUser> {
    return this.http.post<AdminUser>(`${this.apiUrl}/users/${id}/activate`, {});
  }

  deactivateAdminUser(id: number): Observable<AdminUser> {
    return this.http.post<AdminUser>(`${this.apiUrl}/users/${id}/deactivate`, {});
  }

  sendInvitation(id: number): Observable<void> {
    return this.http.post<void>(`${this.apiUrl}/users/${id}/send-invitation`, {});
  }

  sendPasswordReset(id: number): Observable<void> {
    return this.http.post<void>(`${this.apiUrl}/users/${id}/send-password-reset`, {});
  }

  // Admin Groups
  getAllAdminGroups(params: PaginationParams): Observable<PagedResponse<AdminGroup>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<AdminGroup>>(`${this.apiUrl}/groups/`, { params: httpParams });
  }

  getAdminGroup(id: number): Observable<AdminGroup> {
    return this.http.get<AdminGroup>(`${this.apiUrl}/groups/${id}/`);
  }

  createAdminGroup(group: Partial<AdminGroup>): Observable<AdminGroup> {
    return this.http.post<AdminGroup>(`${this.apiUrl}/groups/`, group);
  }

  updateAdminGroup(id: number, group: Partial<AdminGroup>): Observable<AdminGroup> {
    return this.http.patch<AdminGroup>(`${this.apiUrl}/groups/${id}/`, group);
  }

  deleteAdminGroup(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/groups/${id}/`);
  }

  getGroupMembers(groupId: number): Observable<AdminUser[]> {
    return this.http.get<AdminUser[]>(`${this.apiUrl}/groups/${groupId}/members/`);
  }

  addMemberToGroup(groupId: number, userId: number): Observable<void> {
    return this.http.post<void>(`${this.apiUrl}/groups/${groupId}/members/`, { user_id: userId });
  }

  removeMemberFromGroup(groupId: number, userId: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/groups/${groupId}/members/${userId}`);
  }

  changePassword(oldPassword: string, newPassword: string): Observable<any> {
    return this.http.post(`${this.apiUrl}/change-password/`, {
      old_password: oldPassword,
      new_password: newPassword
    });
  }

  updateMe(user: Partial<AdminUser>): Observable<AdminUser> {
    return this.http.patch<AdminUser>(`${this.apiUrl}/me/`, user);
  }

  getMe(): Observable<AdminUser> {
    return this.http.get<AdminUser>(`${this.apiUrl}/me/`);
  }
}