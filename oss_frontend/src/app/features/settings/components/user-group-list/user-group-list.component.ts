import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { UserGroupService } from '../../../../shared/services/user-group.service';
import { UserGroup } from '../../../../shared/models/user-group.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';
import {TranslatePipe} from '@ngx-translate/core';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-user-group-list',
  templateUrl: './user-group-list.component.html',
  imports: [
    TranslatePipe,
    DatePipe,
  ],
  styleUrls: ['./user-group-list.component.scss'],
})
export class UserGroupListComponent implements OnInit {
  groups: UserGroup[] = [];
  totalItems = 0;
  currentPage = 1;
  pageSize = 10;
  loading = false;
  error: string | null = null;
  Math = Math;  // Make Math available in template

  constructor(
    private userGroupService: UserGroupService,
    private router: Router
  ) { }

  ngOnInit(): void {
    this.loadGroups();
  }

  loadGroups(): void {
    this.loading = true;
    this.error = null;

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.userGroupService.getAllGroups(params).subscribe({
      next: (response: PagedResponse<UserGroup>) => {
        // Load parent groups for each group
        this.groups = response.results;
        this.loadParentGroups();
        this.totalItems = response.count;
        this.loading = false;
      },
      error: (error) => {
        this.error = 'Failed to load user groups';
        this.loading = false;
      }
    });
  }

  private loadParentGroups(): void {
    // Create a map of all groups by ID for quick lookup
    const groupsMap = new Map<number, UserGroup>();
    this.groups.forEach(group => groupsMap.set(group.id, group));

    // Update parent references to use the full group object
    this.groups.forEach(group => {
      if (group.parent_id) {
        const parentGroup = groupsMap.get(group.parent_id);
        if (parentGroup) {
          group.parent = parentGroup;
        }
      }
    });
  }

  onPageChange(page: number): void {
    this.currentPage = page;
    this.loadGroups();
  }

  onCreateGroup(): void {
    this.router.navigate(['/settings/user-groups/create']);
  }

  onEditGroup(id: number): void {
    this.router.navigate(['/settings/user-groups', id, 'edit']);
  }

  onDeleteGroup(id: number): void {
    if (confirm('Are you sure you want to delete this group?')) {
      this.userGroupService.deleteGroup(id).subscribe({
        next: () => {
          this.loadGroups();
        },
        error: (error) => {
          this.error = 'Failed to delete user group';
        }
      });
    }
  }
}
