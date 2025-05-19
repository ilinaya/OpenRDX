import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';
import { environment } from '../../../environments/environment';
import { UserIdentifierType } from '../models/user-identifier-type.model';

@Injectable({
  providedIn: 'root'
})
export class UserIdentifierTypeService {
  private apiUrl = `${environment.apiUrl}/users/identifier-types`;

  constructor(private http: HttpClient) {}

  getIdentifierTypes(): Observable<UserIdentifierType[]> {
    return this.http.get<UserIdentifierType[]>(`${this.apiUrl}/list_all/`);
  }
}
