import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { Nas } from '../../../../shared/models/nas.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-nas-list',
  templateUrl: './nas-list.component.html',
  styleUrls: ['./nas-list.component.scss']
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
}
