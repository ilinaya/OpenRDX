import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { RadiusAttribute, RadiusAttributeCreate, RadiusAttributeUpdate } from '../models/radius-attribute.model';

@Injectable({
  providedIn: 'root'
})
export class RadiusAttributeService {
  private apiUrl = `${environment.apiUrl}/radius/attributes`;

  constructor(private http: HttpClient) { }

  getAttributesByGroup(groupId: number): Observable<RadiusAttribute[]> {
    return this.http.get<RadiusAttribute[]>(`${this.apiUrl}/?group=${groupId}`);
  }

  getAttribute(id: number): Observable<RadiusAttribute> {
    return this.http.get<RadiusAttribute>(`${this.apiUrl}/${id}/`);
  }

  createAttribute(attribute: RadiusAttributeCreate): Observable<RadiusAttribute> {
    return this.http.post<RadiusAttribute>(this.apiUrl, attribute);
  }

  updateAttribute(id: number, attribute: RadiusAttributeUpdate): Observable<RadiusAttribute> {
    return this.http.patch<RadiusAttribute>(`${this.apiUrl}/${id}/`, attribute);
  }

  deleteAttribute(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${id}/`);
  }
} 