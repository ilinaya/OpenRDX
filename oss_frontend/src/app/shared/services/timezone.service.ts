import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { Timezone } from '../models/timezone.model';
import { environment } from '../../../environments/environment';

@Injectable({ providedIn: 'root' })
export class TimezoneService {
  private apiUrl = `${environment.apiUrl}/shared/timezones/`;

  constructor(private http: HttpClient) {}

  getTimezones(): Observable<Timezone[]> {
    return this.http.get<Timezone[]>(this.apiUrl);
  }
}
