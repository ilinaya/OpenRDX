import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, BehaviorSubject } from 'rxjs';
import { tap, map } from 'rxjs/operators';
import { Router } from '@angular/router';
import { environment } from '../../../environments/environment';

interface TokenResponse {
  refresh: string;
  access: string;
}

interface User {
  id: number;
  email: string;
  username?: string;
  first_name?: string;
  last_name?: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient, private router: Router) {
    // Check if user is already logged in
    const token = localStorage.getItem('jwt_token');
    const user = localStorage.getItem('current_user');
    if (token && user) {
      this.currentUserSubject.next(JSON.parse(user));
    }
  }

  login(email: string, password: string): Observable<TokenResponse> {
    return this.http.post<TokenResponse>(`${environment.apiUrl}/auth/token/`, { email, password })
      .pipe(
        tap(response => {
          // Store tokens
          localStorage.setItem('jwt_token', response.access);
          localStorage.setItem('refresh_token', response.refresh);

          // Create a basic user object from the email
          const user: User = {
            id: 0, // We don't have the ID yet, will be updated when we get user details
            email: email
          };

          localStorage.setItem('current_user', JSON.stringify(user));
          this.currentUserSubject.next(user);

          // TODO: Fetch user details from API
        })
      );
  }

  logout(): void {
    // Remove token and user info
    localStorage.removeItem('jwt_token');
    localStorage.removeItem('refresh_token');
    localStorage.removeItem('current_user');
    this.currentUserSubject.next(null);
    this.router.navigate(['/login']);
  }

  refreshToken(): Observable<TokenResponse> {
    const refreshToken = localStorage.getItem('refresh_token');

    if (!refreshToken) {
      this.logout();
      return new Observable(observer => {
        observer.error('No refresh token available');
      });
    }

    return this.http.post<TokenResponse>(`${environment.apiUrl}/authentication/token/refresh/`, { refresh: refreshToken })
      .pipe(
        tap(response => {
          localStorage.setItem('jwt_token', response.access);
        })
      );
  }

  isAuthenticated(): boolean {
    return !!localStorage.getItem('jwt_token');
  }

  getCurrentUser(): any {
    return this.currentUserSubject.value;
  }
}
