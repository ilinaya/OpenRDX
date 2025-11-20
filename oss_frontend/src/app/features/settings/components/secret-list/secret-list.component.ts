import { Component, OnInit } from '@angular/core';
import { Router, ActivatedRoute } from '@angular/router';
import { SecretService } from '../../../../shared/services/secret.service';
import { Secret } from '../../../../shared/models/secret.model';
import { PagedResponse, PaginationParams } from '../../../../shared/models/pagination.model';

@Component({
  selector: 'app-secret-list',
  templateUrl: './secret-list.component.html',
  imports: [],
  styleUrls: ['./secret-list.component.scss'],
})
export class SecretListComponent implements OnInit {
  secrets: Secret[] = [];
  loading = false;
  error = '';

  // Pagination properties
  currentPage = 1;
  pageSize = 10;
  totalItems = 0;
  totalPages = 0;

  constructor(
    private secretService: SecretService,
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

    this.secretService.getAllSecrets(params)
      .subscribe({
        next: (response) => {
          this.secrets = response.results;
          this.totalItems = response.count;
          this.totalPages = Math.ceil(this.totalItems / this.pageSize);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load secrets. Please try again later.';
          console.error('Error loading secrets:', err);
          this.loading = false;
        }
      });
  }

  createNewSecret(): void {
    this.router.navigate(['new'], { relativeTo: this.route });
  }

  viewSecretDetails(id: number): void {
    this.router.navigate([id], { relativeTo: this.route });
  }

  editSecret(id: number): void {
    this.router.navigate([id, 'edit'], { relativeTo: this.route });
  }

  deleteSecret(id: number): void {
    if (confirm('Are you sure you want to delete this secret? This action cannot be undone.')) {
      this.secretService.deleteSecret(id)
        .subscribe({
          next: () => {
            // After deletion, check if we need to go to the previous page
            if (this.secrets.length === 1 && this.currentPage > 1) {
              this.changePage(this.currentPage - 1);
            } else {
              this.loadSecrets();
            }
          },
          error: (err) => {
            this.error = 'Failed to delete secret. Please try again later.';
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
