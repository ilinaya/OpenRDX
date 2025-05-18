import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { AdminService } from '../../../../shared/services/admin.service';
import { AdminUser } from '../../../../shared/models/admin.model';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-admin-detail',
  templateUrl: './admin-detail.component.html',
  imports: [
    DatePipe,
  ],
  styleUrls: ['./admin-detail.component.scss'],
})
export class AdminDetailComponent implements OnInit {
  admin: AdminUser | null = null;
  loading = false;
  error = '';

  constructor(
    private adminService: AdminService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    const id = this.route.snapshot.params['id'];
    this.loadAdmin(id);
  }

  loadAdmin(id: number): void {
    this.loading = true;
    this.error = '';

    this.adminService.getAdmin(id)
      .subscribe({
        next: (admin) => {
          this.admin = admin;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load admin user details. Please try again later.';
          console.error('Error loading admin user:', err);
          this.loading = false;
        }
      });
  }

  editAdmin(): void {
    this.router.navigate(['edit'], { relativeTo: this.route });
  }

  deleteAdmin(): void {
    if (confirm('Are you sure you want to delete this admin user?')) {
      this.adminService.deleteAdmin(this.admin!.id)
        .subscribe({
          next: () => {
            this.router.navigate(['../'], { relativeTo: this.route });
          },
          error: (err) => {
            this.error = 'Failed to delete admin user. Please try again later.';
            console.error('Error deleting admin user:', err);
          }
        });
    }
  }

  activateAdmin(): void {
    this.adminService.updateAdmin(this.admin!.id, { is_active: true })
      .subscribe({
        next: (updatedAdmin) => {
          this.admin = updatedAdmin;
        },
        error: (err) => {
          this.error = 'Failed to activate admin user. Please try again later.';
          console.error('Error activating admin user:', err);
        }
      });
  }

  deactivateAdmin(): void {
    this.adminService.updateAdmin(this.admin!.id, { is_active: false })
      .subscribe({
        next: (updatedAdmin) => {
          this.admin = updatedAdmin;
        },
        error: (err) => {
          this.error = 'Failed to deactivate admin user. Please try again later.';
          console.error('Error deactivating admin user:', err);
        }
      });
  }

  sendPasswordReset(): void {
    this.adminService.sendPasswordReset(this.admin!.id)
      .subscribe({
        next: () => {
          alert('Password reset email has been sent.');
        },
        error: (err) => {
          this.error = 'Failed to send password reset email. Please try again later.';
          console.error('Error sending password reset email:', err);
        }
      });
  }
}
