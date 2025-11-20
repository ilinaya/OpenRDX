import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { Nas } from '../../../../shared/models/nas.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';
import {TranslatePipe} from '@ngx-translate/core';

@Component({
  selector: 'app-nas-list',
  templateUrl: './nas-list.component.html',
  imports: [
    TranslatePipe,
  ],
  styleUrls: ['./nas-list.component.scss'],
})
export class NasListComponent implements OnInit {
  nasDevices: Nas[] = [];
  loading = false;
  error = '';

  // Pagination properties
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private nasService: NasService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    // Subscribe to query params to get the page
    this.route.queryParams.subscribe(params => {
      this.currentPage = params['page'] ? parseInt(params['page'], 10) : 1;
      this.loadNasDevices();
    });
  }

  loadNasDevices(): void {
    this.loading = true;
    this.error = '';

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.nasService.getAllNas(params)
      .subscribe({
        next: (response) => {
          this.nasDevices = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load NAS devices. Please try again later.';
          console.error('Error loading NAS devices:', err);
          this.loading = false;
        }
      });
  }

  createNas(): void {
    this.router.navigate(['/devices/nas/new']);
  }

  viewNas(nas: Nas): void {
    this.router.navigate(['/devices/nas', nas.id]);
  }

  editNas(id: number): void {
    this.router.navigate(['/devices/nas', id, 'edit']);
  }

  deleteNas(id: number): void {
    if (confirm('Are you sure you want to delete this NAS device?')) {
      this.nasService.deleteNas(id)
        .subscribe({
          next: () => {
            // After deletion, check if we need to go to the previous page
            if (this.nasDevices.length === 1 && this.currentPage > 1) {
              this.changePage(this.currentPage - 1);
            } else {
              this.loadNasDevices();
            }
          },
          error: (err) => {
            this.error = 'Failed to delete NAS device. Please try again later.';
            console.error('Error deleting NAS device:', err);
          }
        });
    }
  }

  changePage(page: number): void {
    if (page < 1 || page > this.totalPages) {
      return;
    }

    // Update URL with the new page parameter
    this.router.navigate([], {
      relativeTo: this.route,
      queryParams: { page },
      queryParamsHandling: 'merge'
    });
  }

  downloadTemplate(): void {
    this.nasService.downloadNasDeviceTemplate().subscribe({
      next: (blob) => {
        const url = window.URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        link.download = 'nas_devices_template.xlsx';
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
    
    this.nasService.uploadNasDevicesExcel(file).subscribe({
      next: (response) => {
        this.loading = false;
        if (response.success) {
          const message = `Successfully imported ${response.created} device(s).`;
          if (response.errors && response.errors.length > 0) {
            alert(message + '\n\nErrors:\n' + response.errors.join('\n'));
          } else {
            alert(message);
          }
          this.loadNasDevices();
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
