import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { UserService } from '../../../../shared/services/user.service';
import { UserGroup } from '../../../../shared/models/user-group.model';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-user-group-detail',
  templateUrl: './user-group-detail.component.html',
  imports: [
    DatePipe,
  ],
  styleUrls: ['./user-group-detail.component.scss'],
})
export class UserGroupDetailComponent implements OnInit {
  public group: UserGroup | null = null;
  public loading = false;
  public error = '';

  constructor(
    private userService: UserService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    const id = this.route.snapshot.params['id'];
    if (id) {
      this.loadGroup(+id);
    }
  }

  loadGroup(id: number): void {
    this.loading = true;
    this.error = '';

    this.userService.getUserGroup(id)
      .subscribe({
        next: (group) => {
          this.group = group;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load user group details. Please try again later.';
          console.error('Error loading user group:', err);
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
    if (this.group && confirm('Are you sure you want to delete this user group?')) {
      this.userService.deleteUserGroup(this.group.id)
        .subscribe({
          next: () => {
            this.router.navigate(['../'], { relativeTo: this.route });
          },
          error: (err) => {
            this.error = 'Failed to delete user group. Please try again later.';
            console.error('Error deleting user group:', err);
          }
        });
    }
  }

  goBack(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}

