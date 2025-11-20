import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { PaginationParams } from '../../../../shared/models/pagination.model';
import {UserService} from "../../../../shared/services/user.service";
import {UserGroup} from "../../../../shared/models/user-group.model";
import {TranslateModule} from "@ngx-translate/core";

@Component({
  selector: 'app-user-group-list',
  templateUrl: './user-group-list.component.html',
  imports: [
    TranslateModule,
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

  downloadTemplate(): void {
    this.userService.downloadUserGroupTemplate().subscribe({
      next: (blob) => {
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = 'user_groups_template.xlsx';
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        window.URL.revokeObjectURL(url);
      },
      error: (err) => {
        this.error = 'Failed to download template. Please try again later.';
        console.error('Error downloading template:', err);
      }
    });
  }

  onFileSelected(event: Event): void {
    const input = event.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
      const file = input.files[0];
      this.uploadExcel(file);
    }
  }

  uploadExcel(file: File): void {
    this.loading = true;
    this.error = '';
    
    this.userService.uploadUserGroupsExcel(file).subscribe({
      next: (response) => {
        this.loading = false;
        if (response.success) {
          const message = `Successfully imported ${response.created} group(s).`;
          if (response.errors && response.errors.length > 0) {
            alert(message + '\n\nErrors:\n' + response.errors.join('\n'));
          } else {
            alert(message);
          }
          this.loadGroups();
        }
      },
      error: (err) => {
        this.loading = false;
        const errorMessage = err.error?.error || err.error?.message || 'Failed to upload file. Please try again later.';
        this.error = errorMessage;
        console.error('Error uploading file:', err);
        alert('Error uploading file: ' + errorMessage);
      }
    });
  }
}
