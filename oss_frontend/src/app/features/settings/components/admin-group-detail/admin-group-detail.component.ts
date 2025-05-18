import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { AdminService } from '../../../../shared/services/admin.service';
import { AdminGroup, AdminUser } from '../../../../shared/models/admin.model';

@Component({
  selector: 'app-admin-group-detail',
  templateUrl: './admin-group-detail.component.html',
  styleUrls: ['./admin-group-detail.component.scss']
})
export class AdminGroupDetailComponent implements OnInit {
  public group: AdminGroup | null = null;
  public  members: AdminUser[] = [];
  public loading = false;
  public error = '';

  constructor(
    private adminService: AdminService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    const id = this.route.snapshot.params['id'];
    this.loadGroup(id);
  }

  loadGroup(id: number): void {
    this.loading = true;
    this.error = '';

    this.adminService.getAdminGroup(id)
      .subscribe({
        next: (group) => {
          this.group = group;
          this.loadGroupMembers(id);
          console.log('Admin group loaded successfully');
        },
        error: (err) => {
          this.error = 'Failed to load admin group details. Please try again later.';
          console.error('Error loading admin group:', err);
          this.loading = false;
        }
      });
  }

  loadGroupMembers(groupId: number): void {
    this.adminService.getGroupMembers(groupId)
      .subscribe({
        next: (response) => {
          this.members = response;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load group members. Please try again later.';
          console.error('Error loading group members:', err);
          this.loading = false;
        }
      });
  }

  editGroup(): void {
    if (this.group) {
      this.router.navigate(['edit'], { relativeTo: this.route });
    }
  }

  deleteGroup(): void {
    if (confirm('Are you sure you want to delete this admin group?')) {
      this.adminService.deleteAdminGroup(this.group!.id)
        .subscribe({
          next: () => {
            this.router.navigate(['../'], { relativeTo: this.route });
          },
          error: (err) => {
            this.error = 'Failed to delete admin group. Please try again later.';
            console.error('Error deleting admin group:', err);
          }
        });
    }
  }

  removeMember(memberId: number): void {
    if (confirm('Are you sure you want to remove this member from the group?')) {
      this.adminService.removeMemberFromGroup(this.group!.id, memberId)
        .subscribe({
          next: () => {
            this.members = this.members.filter(member => member.id !== memberId);
          },
          error: (err) => {
            this.error = 'Failed to remove member from group. Please try again later.';
            console.error('Error removing member from group:', err);
          }
        });
    }
  }
} 