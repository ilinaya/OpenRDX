import { Component, OnInit } from '@angular/core';
import { FormBuilder, FormGroup, Validators } from '@angular/forms';
import { ActivatedRoute, Router } from '@angular/router';
import { NasService } from '../../../../shared/services/nas.service';
import { Nas, NasGroup, Secret } from '../../../../shared/models/nas.model';
import { forkJoin, of } from 'rxjs';
import { catchError, switchMap } from 'rxjs/operators';

@Component({
  selector: 'app-nas-form',
  templateUrl: './nas-form.component.html',
  styleUrls: ['./nas-form.component.scss']
})
export class NasFormComponent implements OnInit {
  nasForm: FormGroup;
  isEditMode = false;
  nasId: number | null = null;
  loading = false;
  submitting = false;
  error = '';
  nasGroups: NasGroup[] = [];
  secrets: Secret[] = [];

  constructor(
    private fb: FormBuilder,
    private nasService: NasService,
    private route: ActivatedRoute,
    private router: Router
  ) {
    this.nasForm = this.fb.group({
      name: ['', [Validators.required, Validators.maxLength(255)]],
      description: [''],
      ip_address: ['', [Validators.required, Validators.pattern('^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$')]],
      coa_enabled: [false],
      coa_port: [3799, [Validators.min(1), Validators.max(65535)]],
      group_ids: [[]],
      secret_id: [null],
      is_active: [true]
    });
  }

  ngOnInit(): void {
    this.loading = true;

    // Load NAS groups and secrets
    forkJoin({
      groups: this.nasService.getAllNasGroups().pipe(catchError(() => of([]))),
      // We would need to create a service for secrets, but for now we'll use an empty array
      secrets: of([])
    }).subscribe({
      next: (result) => {
        this.nasGroups = result.groups;
        this.secrets = result.secrets;
        
        // Check if we're in edit mode
        const id = this.route.snapshot.paramMap.get('id');
        if (id && id !== 'new') {
          this.isEditMode = true;
          this.nasId = +id;
          this.loadNasDetails(+id);
        } else {
          this.loading = false;
        }
      },
      error: (err) => {
        this.error = 'Failed to load form data. Please try again later.';
        console.error('Error loading form data:', err);
        this.loading = false;
      }
    });
  }

  loadNasDetails(id: number): void {
    this.nasService.getNasById(id)
      .subscribe({
        next: (nas) => {
          // Populate the form with NAS details
          this.nasForm.patchValue({
            name: nas.name,
            description: nas.description,
            ip_address: nas.ip_address,
            coa_enabled: nas.coa_enabled,
            coa_port: nas.coa_port,
            group_ids: nas.groups.map(g => g.id),
            secret_id: nas.secret?.id || null,
            is_active: nas.is_active
          });
          this.loading = false;
        },
        error: (err) => {
          this.error = 'Failed to load NAS details. Please try again later.';
          console.error('Error loading NAS details:', err);
          this.loading = false;
        }
      });
  }

  onSubmit(): void {
    if (this.nasForm.invalid) {
      // Mark all fields as touched to trigger validation messages
      Object.keys(this.nasForm.controls).forEach(key => {
        const control = this.nasForm.get(key);
        control?.markAsTouched();
      });
      return;
    }

    this.submitting = true;
    this.error = '';

    const nasData = this.nasForm.value;

    if (this.isEditMode && this.nasId) {
      // Update existing NAS
      this.nasService.updateNas(this.nasId, nasData)
        .subscribe({
          next: () => {
            this.router.navigate(['/devices/nas', this.nasId]);
          },
          error: (err) => {
            this.error = 'Failed to update NAS. Please try again later.';
            console.error('Error updating NAS:', err);
            this.submitting = false;
          }
        });
    } else {
      // Create new NAS
      this.nasService.createNas(nasData)
        .subscribe({
          next: (nas) => {
            this.router.navigate(['/devices/nas', nas.id]);
          },
          error: (err) => {
            this.error = 'Failed to create NAS. Please try again later.';
            console.error('Error creating NAS:', err);
            this.submitting = false;
          }
        });
    }
  }

  cancel(): void {
    if (this.isEditMode && this.nasId) {
      this.router.navigate(['/devices/nas', this.nasId]);
    } else {
      this.router.navigate(['/devices/nas']);
    }
  }
}