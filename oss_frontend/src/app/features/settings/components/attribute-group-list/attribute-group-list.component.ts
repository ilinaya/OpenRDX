import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { AttributeGroupService } from '../../../../shared/services/attribute-group.service';
import { AttributeGroup } from '../../../../shared/models/attribute-group.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';
import {TranslatePipe} from '@ngx-translate/core';
import {DatePipe} from '@angular/common';

@Component({
  selector: 'app-attribute-group-list',
  templateUrl: './attribute-group-list.component.html',
  imports: [
    TranslatePipe,
    DatePipe,
  ],
  styleUrls: ['./attribute-group-list.component.scss'],
})
export class AttributeGroupListComponent implements OnInit {
  groups: AttributeGroup[] = [];
  totalItems = 0;
  currentPage = 1;
  pageSize = 10;
  loading = false;
  error: string | null = null;
  Math = Math;  // Make Math available in template

  constructor(
    private attributeGroupService: AttributeGroupService,
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

    this.attributeGroupService.getAllGroups(params).subscribe({
      next: (response: PagedResponse<AttributeGroup>) => {
        this.groups = response.results;
        this.totalItems = response.count;
        this.loading = false;
      },
      error: (error) => {
        this.error = 'Failed to load attribute groups';
        this.loading = false;
      }
    });
  }

  onPageChange(page: number): void {
    this.currentPage = page;
    this.loadGroups();
  }

  onCreateGroup(): void {
    this.router.navigate(['/settings/attribute-groups/create']);
  }

  onEditGroup(id: number): void {
    this.router.navigate(['/settings/attribute-groups', id, 'edit']);
  }

  onDeleteGroup(id: number): void {
    if (confirm('Are you sure you want to delete this group?')) {
      this.attributeGroupService.deleteGroup(id).subscribe({
        next: () => {
          this.loadGroups();
        },
        error: (error) => {
          this.error = 'Failed to delete attribute group';
        }
      });
    }
  }
}
