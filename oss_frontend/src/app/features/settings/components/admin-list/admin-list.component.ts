import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { AdminService } from '../../../../shared/services/admin.service';
import { AdminUser } from '../../../../shared/models/admin.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-admin-list',
  templateUrl: './admin-list.component.html',
  styleUrls: ['./admin-list.component.scss']
})
export class AdminListComponent implements OnInit {
  admins: AdminUser[] = [];
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
      this.loadAdmins();
    });
  }

  loadAdmins(): void {
    this.loading = true;
    this.error = '';

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.adminService.getAllAdmins(params)
      .subscribe({
        next: (response) => {
          this.admins = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load admin users. Please try again later.';
          console.error('Error loading admin users:', err);
          this.loading = false;
        }
      });
  }

  createNewAdmin(): void {
    this.router.navigate(['new'], { relativeTo: this.route });
  }

  viewAdminDetails(id: number): void {
    this.router.navigate([id], { relativeTo: this.route });
  }

  editAdmin(id: number): void {
    this.router.navigate([id, 'edit'], { relativeTo: this.route });
  }

  deleteAdmin(id: number): void {
    if (confirm('Are you sure you want to delete this admin user?')) {
      this.adminService.deleteAdmin(id)
        .subscribe({
          next: () => {
            // After deletion, check if we need to go to the previous page
            if (this.admins.length === 1 && this.currentPage > 1) {
              this.changePage(this.currentPage - 1);
            } else {
              this.loadAdmins();
            }
          },
          error: (err) => {
            this.error = 'Failed to delete admin user. Please try again later.';
            console.error('Error deleting admin user:', err);
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

  activateAdmin(id: number): void {
    this.adminService.activateAdminUser(id)
      .subscribe({
        next: () => {
          this.loadAdmins();
        },
        error: (err) => {
          this.error = 'Failed to activate administrator. Please try again later.';
          console.error('Error activating administrator:', err);
        }
      });
  }
  
  deactivateAdmin(id: number): void {
    this.adminService.deactivateAdminUser(id)
      .subscribe({
        next: () => {
          this.loadAdmins();
        },
        error: (err) => {
          this.error = 'Failed to deactivate administrator. Please try again later.';
          console.error('Error deactivating administrator:', err);
        }
      });
  }
  
  sendInvitation(id: number): void {
    this.adminService.sendInvitation(id)
      .subscribe({
        next: () => {
          alert('Invitation sent successfully');
        },
        error: (err) => {
          this.error = 'Failed to send invitation. Please try again later.';
          console.error('Error sending invitation:', err);
        }
      });
  }
  
  sendPasswordReset(id: number): void {
    this.adminService.sendPasswordReset(id)
      .subscribe({
        next: () => {
          alert('Password reset email sent successfully');
        },
        error: (err) => {
          this.error = 'Failed to send password reset email. Please try again later.';
          console.error('Error sending password reset email:', err);
        }
      });
  }
}