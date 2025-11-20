import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { AdminService } from '../../../../shared/services/admin.service';
import { AdminGroup } from '../../../../shared/models/admin.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-admin-group-list',
  templateUrl: './admin-group-list.component.html',
  imports: [],
  styleUrls: ['./admin-group-list.component.scss'],
})
export class AdminGroupListComponent implements OnInit {
  groups: AdminGroup[] = [];
  loading = false;
  error = '';

  // Pagination properties
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private adminService: AdminService,
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

    this.adminService.getAllAdminGroups(params)
      .subscribe({
        next: (response) => {
          this.groups = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load admin groups. Please try again later.';
          console.error('Error loading admin groups:', err);
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
    if (confirm('Are you sure you want to delete this admin group?')) {
      this.adminService.deleteAdminGroup(id)
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
            this.error = 'Failed to delete admin group. Please try again later.';
            console.error('Error deleting admin group:', err);
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
