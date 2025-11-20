import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { ApiKey, ApiKeyCreate } from '../models/api-key.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class ApiKeyService {
  private apiUrl = `${environment.apiUrl}/api-keys/`;

  constructor(private http: HttpClient) { }

  getAllApiKeys(params?: PaginationParams): Observable<PagedResponse<ApiKey>> {
    let httpParams = new HttpParams();
    if (params) {
      httpParams = httpParams
        .set('page', (params.page || 1).toString())
        .set('page_size', (params.page_size || 10).toString());
    }

    return this.http.get<PagedResponse<ApiKey>>(this.apiUrl, { params: httpParams });
  }

  getApiKey(id: number): Observable<ApiKey> {
    return this.http.get<ApiKey>(`${this.apiUrl}${id}/`);
  }

  createApiKey(apiKey: ApiKeyCreate): Observable<ApiKey> {
    return this.http.post<ApiKey>(this.apiUrl, apiKey);
  }

  deleteApiKey(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}${id}/`);
  }

  revokeApiKey(id: number): Observable<ApiKey> {
    return this.http.post<ApiKey>(`${this.apiUrl}${id}/revoke/`, {});
  }

  activateApiKey(id: number): Observable<ApiKey> {
    return this.http.post<ApiKey>(`${this.apiUrl}${id}/activate/`, {});
  }
}

