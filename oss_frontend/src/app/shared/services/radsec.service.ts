import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { PagedResponse, PaginationParams } from '../models/pagination.model';
import {RadSecSource, RadSecSourceCreate, RadSecSourceUpdate} from "../models/radsec.model";

@Injectable({
  providedIn: 'root'
})
export class RadsecService {
  private apiUrl = `${environment.apiUrl}/radsec`;

  constructor(private http: HttpClient) { }

  getAllSources(params: PaginationParams): Observable<PagedResponse<RadSecSource>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<RadSecSource>>(this.apiUrl + '/sources/', { params: httpParams });
  }

  listSources(): Observable<RadSecSource[]> {
    return this.http.get<RadSecSource[]>(`${this.apiUrl}/sources/list_all/`);
  }

  getSource(id: number): Observable<RadSecSource> {
    return this.http.get<RadSecSource>(`${this.apiUrl}/sources/${id}`);
  }

  createSource(source: RadSecSourceCreate): Observable<RadSecSource> {
    return this.http.post<RadSecSource>(this.apiUrl + '/sources/', source);
  }

  updateSource(id: number, source: RadSecSourceUpdate): Observable<RadSecSource> {
    return this.http.patch<RadSecSource>(`${this.apiUrl}/sources/${id}/`, source);
  }

  deleteSource(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/sources/${id}/`);
  }


}
