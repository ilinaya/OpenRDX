import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { DatePipe, NgClass, NgForOf, NgIf } from "@angular/common";
import { TranslatePipe, TranslateService } from "@ngx-translate/core";
import { UserService } from "../../../../shared/services/user.service";
import { User } from "../../../../shared/models/user.model";
import { catchError, finalize } from 'rxjs/operators';
import { of } from 'rxjs';

@Component({
  selector: 'app-user-detail',
  templateUrl: './user-detail.component.html',
  imports: [
    DatePipe,
    NgClass,
    TranslatePipe
  ],
  styleUrls: ['./user-detail.component.scss']
})
export class UserDetailComponent implements OnInit {
  user: User | null = null;
  loading = false;
  error: string | null = null;

  constructor(
    private route: ActivatedRoute,
    private userService: UserService,
    private router: Router,
    private translate: TranslateService
  ) {}

  ngOnInit(): void {
    const userId = this.route.snapshot.paramMap.get('id');
    if (userId) {
      this.loadUser(Number(userId));
    } else {
      this.error = this.translate.instant('users.detail.invalidId');
      this.router.navigate(['/users/users']);
    }
  }

  loadUser(id: number): void {
    this.loading = true;
    this.error = null;

    this.userService.getUser(id).pipe(
      catchError(error => {
        this.error = this.translate.instant('users.detail.loadError');
        console.error('Error loading user:', error);
        return of(null);
      }),
      finalize(() => {
        this.loading = false;
      })
    ).subscribe(user => {
      if (user) {
        this.user = user;
      }
    });
  }

  editUser(): void {
    if (this.user) {
      this.router.navigate(['/users/users', this.user.id, 'edit']);
    }
  }

  deleteUser(): void {
    if (!this.user) return;

    if (confirm(this.translate.instant('users.detail.deleteConfirm'))) {
      this.loading = true;
      this.error = null;

      this.userService.deleteUser(this.user.id).pipe(
        catchError(error => {
          this.error = this.translate.instant('users.detail.deleteError');
          console.error('Error deleting user:', error);
          return of(null);
        }),
        finalize(() => {
          this.loading = false;
        })
      ).subscribe(result => {
        if (result !== null) {
          this.router.navigate(['/users/users']);
        }
      });
    }
  }

  goBack(): void {
    this.router.navigate(['/users/users']);
  }
}
