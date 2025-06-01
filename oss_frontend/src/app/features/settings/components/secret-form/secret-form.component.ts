import { Component, OnInit } from '@angular/core';
import {FormBuilder, FormGroup, ReactiveFormsModule, Validators} from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { SecretService } from '../../../../shared/services/secret.service';
import { Secret } from '../../../../shared/models/secret.model';

@Component({
  selector: 'app-secret-form',
  templateUrl: './secret-form.component.html',
  imports: [
    ReactiveFormsModule,
  ],
  styleUrls: ['./secret-form.component.scss'],
})
export class SecretFormComponent implements OnInit {
  secretForm: FormGroup;
  isEditMode = false;
  loading = false;
  submitting = false;
  error = '';
  secretId?: number;

  constructor(
    private fb: FormBuilder,
    private secretService: SecretService,
    private router: Router,
    private route: ActivatedRoute
  ) {
    this.secretForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      secret: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      source_subnets: [[]]
    });
  }

  ngOnInit(): void {
    this.secretId = this.route.snapshot.params['id'];
    if (this.secretId) {
      this.isEditMode = true;
      this.loadSecret();
    }
  }

  loadSecret(): void {
    if (!this.secretId) return;

    this.loading = true;
    this.secretService.getSecret(this.secretId)
      .subscribe({
        next: (secret) => {
          this.secretForm.patchValue(secret);
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load secret. Please try again later.';
          console.error('Error loading secret:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.secretForm.invalid) {
      return;
    }

    this.submitting = true;
    const secretData = this.secretForm.value;

    const request = this.isEditMode
      ? this.secretService.updateSecret(this.secretId!, secretData)
      : this.secretService.createSecret(secretData);

    request.subscribe({
      next: () => {
        this.router.navigate(['../'], { relativeTo: this.route });
      },
      error: (err) => {
        this.error = `Failed to ${this.isEditMode ? 'update' : 'create'} secret. Please try again later.`;
        console.error(`Error ${this.isEditMode ? 'updating' : 'creating'} secret:`, err);
        this.submitting = false;
      }
    });
  }

  cancel(): void {
    this.router.navigate(['../'], { relativeTo: this.route });
  }
}
