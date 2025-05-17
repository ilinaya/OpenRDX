import { Injectable } from '@angular/core';
import {
  HttpRequest,
  HttpHandler,
  HttpEvent,
  HttpInterceptor,
  HttpErrorResponse
} from '@angular/common/http';
import { Observable, throwError } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { Router } from '@angular/router';
import { environment } from '../../../environments/environment';

@Injectable()
export class AuthInterceptor implements HttpInterceptor {
  private authEndpoints = [
    `${environment.apiUrl}/auth/token/`,
    `${environment.apiUrl}/auth/token/refresh/`,
    `${environment.apiUrl}/auth/token/verify/`,
    '/api/admin-users/login/'
  ];

  constructor(private router: Router) {}

  intercept(request: HttpRequest<unknown>, next: HttpHandler): Observable<HttpEvent<unknown>> {
    // Skip adding Authorization header for auth endpoints
    if (this.authEndpoints.some(endpoint => request.url.includes(endpoint))) {
      return next.handle(request);
    }

    const token = localStorage.getItem('jwt_token');
    
    if (token) {
      request = request.clone({
        setHeaders: {
          Authorization: `Bearer ${token}`
        }
      });
    }
    
    return next.handle(request).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 401) {
          // Clear token and redirect to login
          localStorage.removeItem('jwt_token');
          this.router.navigate(['/login']);
        }
        return throwError(() => error);
      })
    );
  }
} 