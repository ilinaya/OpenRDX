import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { DatePipe, NgClass, NgForOf } from '@angular/common';
import {User} from "../../../../shared/models/user.model";
import {UserService} from "../../../../shared/services/user.service";
import {TranslatePipe, TranslateService} from '@ngx-translate/core';

@Component({
  selector: 'app-user-list',
  templateUrl: './user-list.component.html',
  styleUrls: ['./user-list.component.scss'],
  imports: [DatePipe, NgClass, TranslatePipe]
})
export class UserListComponent implements OnInit {
  users: User[] = [];
  loading = false;
  error: string | null = null;
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private userService: UserService,
    private router: Router,
    private route: ActivatedRoute,
    private translate: TranslateService
  ) {}

  ngOnInit(): void {
    this.route.queryParams.subscribe(params => {
      this.currentPage = params['page'] ? parseInt(params['page'], 10) : 1;
      this.loadUsers();
    });
  }

  loadUsers(): void {
    this.loading = true;
    this.error = null;

    this.userService.getUsers(this.currentPage, this.pageSize).subscribe({
      next: (response) => {
        this.users = response.results;
        this.totalItems = response.count;
        this.totalPages = Math.ceil(this.totalItems / this.pageSize);
        this.loading = false;
      },
      error: (error) => {
        this.loading = false;
        this.translate.get('users.list.loadError').subscribe((msg: string) => {
          this.error = msg;
        });
      }
    });
  }

  createUser(): void {
    this.router.navigate(['/users/users/new']);
  }

  viewUser(user: User): void {
    this.router.navigate(['/users', user.id]);
  }

  editUser(user: User): void {
    this.router.navigate(['/users/users/', user.id, 'edit']);
  }

  deleteUser(user: User): void {
    if (confirm(this.translate.instant('users.list.deleteConfirm'))) {
      this.userService.deleteUser(user.id).subscribe({
        next: () => {
          if (this.users.length === 1 && this.currentPage > 1) {
            this.changePage(this.currentPage - 1);
          } else {
            this.loadUsers();
          }
        },
        error: (error) => {
          this.translate.get('users.list.deleteError').subscribe((msg: string) => {
            this.error = msg;
          });
        }
      });
    }
  }

  changePage(page: number): void {
    if (page < 1 || page > this.totalPages) {
      return;
    }

    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { page },
      queryParamsHandling: 'merge'
    });
  }
}
