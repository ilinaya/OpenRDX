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

  downloadTemplate(): void {
    this.nasService.downloadNasGroupTemplate().subscribe({
      next: (blob) => {
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = 'nas_groups_template.xlsx';
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
    
    this.nasService.uploadNasGroupsExcel(file).subscribe({
      next: (response) => {
        this.loading = false;
        if (response.success) {
          const message = `Successfully imported ${response.created} group(s).`;
          if (response.errors && response.errors.length > 0) {
            alert(message + '\n\nErrors:\n' + response.errors.join('\n'));
          } else {
            alert(message);
          }
          this.loadNasGroups();
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
