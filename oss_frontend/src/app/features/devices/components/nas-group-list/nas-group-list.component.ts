import {Component, OnInit} from '@angular/core';
import {Router} from '@angular/router';
import {NasService} from '../../../../shared/services/nas.service';
import {NasGroup} from '../../../../shared/models/nas.model';
import {TranslatePipe} from '@ngx-translate/core';
import {PaginationParams} from "../../../../shared/models/pagination.model";

@Component({
  selector: 'app-nas-group-list',
  templateUrl: './nas-group-list.component.html',
  styleUrls: ['./nas-group-list.component.scss'],
  imports: [
    TranslatePipe,
  ],
})
export class NasGroupListComponent implements OnInit {
  nasGroups: NasGroup[] = [];
  loading = false;
  error = '';

  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private nasService: NasService,
    private router: Router,
  ) {
  }

  ngOnInit(): void {
    this.loadNasGroups();
  }

  loadNasGroups(): void {
    this.loading = true;
    this.error = '';

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.nasService.getAllNasGroups(params)
      .subscribe({
        next: (groups) => {
          this.nasGroups = groups.results;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load NAS groups. Please try again later.';
          console.error('Error loading NAS groups:', err);
          this.loading = false;
        },
      });
  }

  viewNasInGroup(groupId: number): void {
    this.router.navigate(['/devices/nas'], {queryParams: {group: groupId}});
  }

  editGroup(id: number, event: Event): void {
    event.stopPropagation();
    this.router.navigate(['/devices/groups', id, 'edit']);
  }

  deleteGroup(id: number, event: Event): void {
    event.stopPropagation();
    if (confirm('Are you sure you want to delete this NAS group?')) {
      this.nasService.deleteNasGroup(id)
        .subscribe({
          next: () => {
            this.loadNasGroups();
          },
          error: (err) => {
            this.error = 'Failed to delete NAS group. Please try again later.';
            console.error('Error deleting NAS group:', err);
          },
        });
    }
  }

  createNewGroup(): void {
    this.router.navigate(['/devices/groups/new']);
  }
}
