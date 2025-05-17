import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { AttributeGroup, AttributeGroupCreate, AttributeGroupUpdate } from '../models/attribute-group.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class AttributeGroupService {
  private apiUrl = `${environment.apiUrl}/radius/attribute-groups`;

  constructor(private http: HttpClient) { }

  getAllGroups(params: PaginationParams): Observable<PagedResponse<AttributeGroup>> {
    return this.http.get<PagedResponse<AttributeGroup>>(this.apiUrl, { params: params as any });
  }

  getGroup(id: number): Observable<AttributeGroup> {
    return this.http.get<AttributeGroup>(`${this.apiUrl}/${id}/`);
  }

  createGroup(group: AttributeGroupCreate): Observable<AttributeGroup> {
    return this.http.post<AttributeGroup>(this.apiUrl, group);
  }

  updateGroup(id: number, group: AttributeGroupUpdate): Observable<AttributeGroup> {
    return this.http.patch<AttributeGroup>(`${this.apiUrl}/${id}/`, group);
  }

  deleteGroup(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${id}/`);
  }
} 