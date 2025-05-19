import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { PaginationParams } from '../../../../shared/models/pagination.model';
import {DatePipe} from '@angular/common';
import {UserService} from "../../../../shared/services/user.service";
import {UserGroup} from "../../../../shared/models/user-group.model";

@Component({
  selector: 'app-user-group-list',
  templateUrl: './user-group-list.component.html',
  imports: [
    DatePipe,
  ],
  styleUrls: ['./user-group-list.component.scss'],
})
export class UserGroupListComponent implements OnInit {
  groups: UserGroup[] = [];
  loading = false;
  error = '';

  // Pagination properties
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private userService: UserService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    // Subscribe to query params to get the page
    this.route.queryParams.subscribe(params => {
      this.currentPage = params['page'] ? parseInt(params['page'], 10) : 1;
      this.loadGroups();
    });
  }

  loadGroups(): void {
    this.loading = true;
    this.error = '';

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.userService.getUserGroups(params)
      .subscribe({
        next: (response) => {
          this.groups = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load user groups. Please try again later.';
          console.error('Error loading user groups:', err);
          this.loading = false;
        }
      });
  }

  createNewGroup(): void {
    this.router.navigate(['new'], { relativeTo: this.route });
  }

  viewGroupDetails(id: number): void {
    this.router.navigate([id], { relativeTo: this.route });
  }

  editGroup(id: number): void {
    this.router.navigate([id, 'edit'], { relativeTo: this.route });
  }

  deleteGroup(id: number): void {
    if (confirm('Are you sure you want to delete this user group?')) {
      this.userService.deleteUserGroup(id)
        .subscribe({
          next: () => {
            // After deletion, check if we need to go to the previous page
            if (this.groups.length === 1 && this.currentPage > 1) {
              this.changePage(this.currentPage - 1);
            } else {
              this.loadGroups();
            }
          },
          error: (err) => {
            this.error = 'Failed to delete user group. Please try again later.';
            console.error('Error deleting user group:', err);
          }
        });
    }
  }

  changePage(page: number): void {
    if (page >= 1 && page <= this.totalPages) {
      this.router.navigate([], {
        relativeTo: this.route,
        queryParams: { page: page }
      });
    }
  }
}
