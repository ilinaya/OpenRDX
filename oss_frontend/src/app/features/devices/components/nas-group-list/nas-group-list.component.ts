import {Component, OnInit} from '@angular/core';
import {Router} from '@angular/router';
import {NasService} from '../../../../shared/services/nas.service';
import {NasGroup} from '../../../../shared/models/nas.model';
import {TranslatePipe} from '@ngx-translate/core';

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
  treeView = true;
  loading = false;
  error = '';

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

    if (this.treeView) {
      this.nasService.getNasGroupTree()
        .subscribe({
          next: (groups) => {
            this.nasGroups = groups;
            this.loading = false;
          },
          error: (err) => {
            this.error = 'Failed to load NAS groups. Please try again later.';
            console.error('Error loading NAS groups:', err);
            this.loading = false;
          },
        });
    } else {
      this.nasService.getAllNasGroups()
        .subscribe({
          next: (groups) => {
            this.nasGroups = groups;
            this.loading = false;
          },
          error: (err) => {
            this.error = 'Failed to load NAS groups. Please try again later.';
            console.error('Error loading NAS groups:', err);
            this.loading = false;
          },
        });
    }
  }

  toggleView(): void {
    this.treeView = !this.treeView;
    this.loadNasGroups();
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
