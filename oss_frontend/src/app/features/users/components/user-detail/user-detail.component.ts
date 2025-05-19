import { Component, OnInit } from '@angular/core';
import {ActivatedRoute, Router} from '@angular/router';
import {DatePipe, NgClass, NgForOf, NgIf} from "@angular/common";
import {TranslatePipe} from "@ngx-translate/core";
import {UserService} from "../../../../shared/services/user.service";
import {User} from "../../../../shared/models/user.model";

@Component({
  selector: 'app-user-detail',
  templateUrl: './user-detail.component.html',
  imports: [
    DatePipe,
    NgClass,
    NgForOf,
    TranslatePipe
  ],
  styleUrls: ['./user-detail.component.scss']
})
export class UserDetailComponent implements OnInit {
  user: User | null = null;
  loading = false;
  error = '';

  constructor(
    private route: ActivatedRoute,
    private userService: UserService,
    private router: Router
  ) {}

  ngOnInit(): void {
    const userId = this.route.snapshot.paramMap.get('id');
    if (userId) {
      this.loadUser(Number(userId));
    }
  }

  loadUser(id: number): void {
    this.userService.getUser(id).subscribe(user => {
      this.user = user;
    });
  }

  editUser(): void {
    if (this.user) {
      this.router.navigate(['/users/users', this.user.id, 'edit']);
    }
  }

  deleteUser(): void {
    if (!this.user) return;

    if (confirm('Are you sure you want to delete this User?')) {
      this.userService.deleteUser(this.user.id)
        .subscribe({
          next: () => {
            this.router.navigate(['/users/users']);
          },
          error: (err) => {
            this.error = 'Failed to delete User. Please try again later.';
            console.error('Error deleting User:', err);
          }
        });
    }
  }

  goBack(): void {
    window.history.back();
  }
}
