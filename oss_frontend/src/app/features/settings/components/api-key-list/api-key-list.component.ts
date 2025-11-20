import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { ApiKeyService } from '../../../../shared/services/api-key.service';
import { ApiKey } from '../../../../shared/models/api-key.model';
import { CommonModule } from '@angular/common';
import { TranslateModule } from '@ngx-translate/core';
import { DatePipe } from '@angular/common';

@Component({
  selector: 'app-api-key-list',
  templateUrl: './api-key-list.component.html',
  styleUrls: ['./api-key-list.component.scss'],
  standalone: true,
  imports: [
    CommonModule,
    TranslateModule,
    DatePipe,
  ],
})
export class ApiKeyListComponent implements OnInit {
  apiKeys: ApiKey[] = [];
  loading = false;
  error = '';
  currentPage = 1;
  totalPages = 1;
  totalItems = 0;
  pageSize = 10;
  selectedApiKey: ApiKey | null = null;
  copied = false;

  constructor(
    private apiKeyService: ApiKeyService,
    private router: Router
  ) {}

  ngOnInit(): void {
    this.loadApiKeys();
  }

  loadApiKeys(): void {
    this.loading = true;
    this.error = '';

    this.apiKeyService.getAllApiKeys({
      page: this.currentPage,
      page_size: this.pageSize
    }).subscribe({
      next: (response) => {
        this.apiKeys = response.results || [];
        this.totalItems = response.count || 0;
        this.totalPages = Math.ceil(this.totalItems / this.pageSize);
        this.loading = false;
      },
      error: (err) => {
        this.error = 'Failed to load API keys. Please try again later.';
        console.error('Error loading API keys:', err);
        this.loading = false;
      },
    });
  }

  createApiKey(): void {
    this.router.navigate(['/settings/api-keys/new']);
  }

  deleteApiKey(apiKey: ApiKey): void {
    if (confirm(`Are you sure you want to delete API key "${apiKey.name}"?`)) {
      this.apiKeyService.deleteApiKey(apiKey.id).subscribe({
        next: () => {
          this.loadApiKeys();
        },
        error: (err) => {
          this.error = 'Failed to delete API key. Please try again later.';
          console.error('Error deleting API key:', err);
        },
      });
    }
  }

  revokeApiKey(apiKey: ApiKey): void {
    if (confirm(`Are you sure you want to revoke API key "${apiKey.name}"?`)) {
      this.apiKeyService.revokeApiKey(apiKey.id).subscribe({
        next: () => {
          this.loadApiKeys();
        },
        error: (err) => {
          this.error = 'Failed to revoke API key. Please try again later.';
          console.error('Error revoking API key:', err);
        },
      });
    }
  }

  activateApiKey(apiKey: ApiKey): void {
    this.apiKeyService.activateApiKey(apiKey.id).subscribe({
      next: () => {
        this.loadApiKeys();
      },
      error: (err) => {
        this.error = 'Failed to activate API key. Please try again later.';
        console.error('Error activating API key:', err);
      },
    });
  }

  viewApiKey(apiKey: ApiKey): void {
    this.selectedApiKey = apiKey;
    this.copied = false;
  }

  closeModal(): void {
    this.selectedApiKey = null;
    this.copied = false;
  }

  copyToClipboard(text: string): void {
    navigator.clipboard.writeText(text).then(() => {
      this.copied = true;
      setTimeout(() => {
        this.copied = false;
      }, 2000);
    }).catch(err => {
      console.error('Failed to copy to clipboard:', err);
      this.error = 'Failed to copy to clipboard';
    });
  }

  onPageChange(page: number): void {
    this.currentPage = page;
    this.loadApiKeys();
  }
}

