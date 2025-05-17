import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { SecretService } from '../../../../shared/services/secret.service';
import { Secret } from '../../../../shared/models/secret.model';

@Component({
  selector: 'app-secret-detail',
  templateUrl: './secret-detail.component.html',
  styleUrls: ['./secret-detail.component.scss']
})
export class SecretDetailComponent implements OnInit {
  secret?: Secret;
  loading = false;
  error = '';
  showSecret = false;

  constructor(
    private secretService: SecretService,
    private router: Router,
    private route: ActivatedRoute
  ) { }

  ngOnInit(): void {
    const id = this.route.snapshot.params['id'];
    if (id) {
      this.loadSecret(id);
    }
  }

  loadSecret(id: number): void {
    this.loading = true;
    this.secretService.getSecret(id)
      .subscribe({
        next: (secret) => {
          this.secret = secret;
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load secret. Please try again later.';
          console.error('Error loading secret:', err);
          this.loading = false;
        }
      });
  }

  editSecret(): void {
    if (this.secret) {
      this.router.navigate(['edit'], { relativeTo: this.route });
    }
  }

  deleteSecret(): void {
    if (this.secret && confirm('Are you sure you want to delete this secret? This action cannot be undone.')) {
      this.secretService.deleteSecret(this.secret.id)
        .subscribe({
          next: () => {
            this.router.navigate(['../'], { relativeTo: this.route });
          },
          error: (err) => {
            this.error = 'Failed to delete secret. Please try again later.';
            console.error('Error deleting secret:', err);
          }
        });
    }
  }

  toggleSecretVisibility(): void {
    this.showSecret = !this.showSecret;
  }
} 