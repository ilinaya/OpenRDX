import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { Secret, SecretCreate, SecretUpdate } from '../models/secret.model';
import { PagedResponse, PaginationParams } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class SecretService {
  private apiUrl = `${environment.apiUrl}/radius/secrets`;

  constructor(private http: HttpClient) { }

  getAllSecrets(params: PaginationParams): Observable<PagedResponse<Secret>> {
    const httpParams = new HttpParams()
      .set('page', (params.page || 1).toString())
      .set('page_size', (params.page_size || 10).toString());

    return this.http.get<PagedResponse<Secret>>(this.apiUrl, { params: httpParams });
  }

  getSecret(id: number): Observable<Secret> {
    return this.http.get<Secret>(`${this.apiUrl}/${id}`);
  }

  createSecret(secret: SecretCreate): Observable<Secret> {
    return this.http.post<Secret>(this.apiUrl, secret);
  }

  updateSecret(id: number, secret: SecretUpdate): Observable<Secret> {
    return this.http.patch<Secret>(`${this.apiUrl}/${id}`, secret);
  }

  deleteSecret(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${id}`);
  }

  encryptSecret(id: number): Observable<Secret> {
    return this.http.post<Secret>(`${this.apiUrl}/${id}/encrypt`, {});
  }

  decryptSecret(id: number): Observable<Secret> {
    return this.http.post<Secret>(`${this.apiUrl}/${id}/decrypt`, {});
  }
} 