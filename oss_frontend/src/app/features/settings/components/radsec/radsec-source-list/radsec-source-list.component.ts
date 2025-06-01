import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import {DatePipe} from '@angular/common';
import {RadSecSource} from "../../../../../shared/models/radsec.model";
import {RadsecService} from "../../../../../shared/services/radsec.service";
import {PaginationParams} from "../../../../../shared/models/pagination.model";

@Component({
  selector: 'app-radsec-source-list',
  templateUrl: './radsec-source-list.component.html',
  imports: [
    DatePipe,
  ],
  styleUrls: ['./radsec-source-list.component.scss'],
})
export class RadsecSourceListComponent implements OnInit {
  sources: RadSecSource[] = [];
  loading = false;
  error = '';

  // Pagination properties
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private radsecService: RadsecService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    // Subscribe to query params to get the page
    this.route.queryParams.subscribe(params => {
      this.currentPage = params['page'] ? parseInt(params['page'], 10) : 1;
      this.loadSecrets();
    });
  }

  loadSecrets(): void {
    this.loading = true;
    this.error = '';

    const params: PaginationParams = {
      page: this.currentPage,
      page_size: this.pageSize
    };

    this.radsecService.getAllSources(params)
      .subscribe({
        next: (response) => {
          this.sources = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load sources. Please try again later.';
          console.error('Error loading secrets:', err);
          this.loading = false;
        }
      });
  }

  createNewSource(): void {
    this.router.navigate(['new'], { relativeTo: this.route });
  }

  viewSourceDetails(id: number): void {
    this.router.navigate([id], { relativeTo: this.route });
  }

  editSource(id: number): void {
    this.router.navigate([id, 'edit'], { relativeTo: this.route });
  }

  deleteSource(id: number): void {
    if (confirm('Are you sure you want to delete this radsec source? This action cannot be undone.')) {
      this.radsecService.deleteSource(id)
        .subscribe({
          next: () => {
            // After deletion, check if we need to go to the previous page
            if (this.sources.length === 1 && this.currentPage > 1) {
              this.changePage(this.currentPage - 1);
            } else {
              this.loadSecrets();
            }
          },
          error: (err) => {
            this.error = 'Failed to delete radsec seource. Please try again later.';
            console.error('Error deleting secret:', err);
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
}
