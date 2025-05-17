import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { Vendor } from '../models/vendor.model';
import { PagedResponse } from '../models/pagination.model';

@Injectable({
  providedIn: 'root'
})
export class VendorService {
  private apiUrl = `${environment.apiUrl}/nas/vendors`;

  constructor(private http: HttpClient) {}

  getAllVendors(params: { page?: number; page_size?: number } = {}): Observable<PagedResponse<Vendor>> {
    let httpParams = new HttpParams();
    if (params.page) {
      httpParams = httpParams.set('page', params.page.toString());
    }
    if (params.page_size) {
      httpParams = httpParams.set('page_size', params.page_size.toString());
    }
    return this.http.get<PagedResponse<Vendor>>(this.apiUrl, { params: httpParams });
  }

  getAllVendorsList(): Observable<Vendor[]> {
    return this.http.get<Vendor[]>(`${this.apiUrl}/list_all/`);
  }

  getVendor(id: number): Observable<Vendor> {
    return this.http.get<Vendor>(`${this.apiUrl}/${id}/`);
  }

  createVendor(vendor: Partial<Vendor>): Observable<Vendor> {
    return this.http.post<Vendor>(this.apiUrl, vendor);
  }

  updateVendor(id: number, vendor: Partial<Vendor>): Observable<Vendor> {
    return this.http.put<Vendor>(`${this.apiUrl}/${id}/`, vendor);
  }

  partialUpdateVendor(id: number, vendor: Partial<Vendor>): Observable<Vendor> {
    return this.http.patch<Vendor>(`${this.apiUrl}/${id}/`, vendor);
  }

  deleteVendor(id: number): Observable<void> {
    return this.http.delete<void>(`${this.apiUrl}/${id}/`);
  }
} 